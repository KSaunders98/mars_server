use std::sync::{Arc, Mutex};

use diesel::prelude::*;
use diesel::mysql::MysqlConnection;
use dotenv::dotenv;
use gdal::raster::dataset::Dataset;
use rayon::prelude::*;

use common::models::Tile;

const LEFT_LONG: f64 = -180.0;
const RIGHT_LONG: f64 = 180.0;
const TOP_LAT: f64 = 90.0;
const BOTTOM_LAT: f64 = -90.0;
const TILE_SIZE_PIXELS: usize = 256;

const BATCH_SIZE: usize = 10;

const NUM_TILES_LONG: usize = (RIGHT_LONG - LEFT_LONG) as usize;
const NUM_TILES_LAT: usize = (TOP_LAT - BOTTOM_LAT) as usize;

pub fn establish_connection() -> MysqlConnection {
    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in the environment or .env file");
    MysqlConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
}

fn create_tiles(thread_num: usize, num_threads: usize, progress: Arc<Mutex<usize>>) {
    use common::schema::tiles;

    // each thread has its own handles to the resources
    let connection = establish_connection();
    let elevation = Dataset::open(std::path::Path::new("datasets/Mars_HRSC_MOLA_BlendDEM_Global_200mp_v2.tif")).expect("Unable to open elevation dataset");
    let elevation_band = elevation.rasterband(1).expect("Unable to open elevation rasterband");
    let (elev_size_x, elev_size_y) = elevation.size();
    let imagery = Dataset::open(std::path::Path::new("datasets/Mars_Viking_MDIM21_ClrMosaic_global_232m.tif")).expect("Unable to open imagery dataset");
    let red_band = imagery.rasterband(1).expect("Unable to open imagery red band");
    let green_band = imagery.rasterband(2).expect("Unable to open imagery green band");
    let blue_band = imagery.rasterband(3).expect("Unable to open imagery blue band");
    let (img_size_x, img_size_y) = imagery.size();
    let mut batch = Vec::with_capacity(BATCH_SIZE);

    for tile_y in (thread_num..NUM_TILES_LAT).step_by(num_threads) {
        let percent_y = (tile_y as f64) / (NUM_TILES_LAT as f64);
        let next_percent_y = ((tile_y + 1) as f64) / (NUM_TILES_LAT as f64);
        let elev_y = (((elev_size_y - 1) as f64) * percent_y) as usize;
        let elev_next_y = (((elev_size_y - 1) as f64) * next_percent_y) as usize;
        let elev_size_y = elev_next_y - elev_y + 1;
        let img_y = (((img_size_y - 1) as f64) * percent_y) as usize;
        let img_next_y = (((img_size_y - 1) as f64) * next_percent_y) as usize;
        let img_size_y = img_next_y - img_y + 1;
        let latitude = TOP_LAT + (BOTTOM_LAT - TOP_LAT) * percent_y;

        for tile_x in 0..NUM_TILES_LONG {
            let percent_x = (tile_x as f64) / (NUM_TILES_LONG as f64);
            let next_percent_x = ((tile_x + 1) as f64) / (NUM_TILES_LONG as f64);
            let elev_x = (((elev_size_x - 1) as f64) * percent_x) as usize;
            let elev_next_x = (((elev_size_x - 1) as f64) * next_percent_x) as usize;
            let elev_size_x = elev_next_x - elev_x + 1;
            let img_x = (((img_size_x - 1) as f64) * percent_x) as usize;
            let img_next_x = (((img_size_x - 1) as f64) * next_percent_x) as usize;
            let img_size_x = img_next_x - img_x + 1;
            let longitude = LEFT_LONG + (RIGHT_LONG - LEFT_LONG) * percent_x;

            let id = tile_y * NUM_TILES_LONG + tile_x;
            let mut tile = Tile {
                id: id as u32,
                latitude: latitude.round() as i32,
                longitude: longitude.round() as i32,
                elevation_data: Vec::with_capacity(TILE_SIZE_PIXELS * TILE_SIZE_PIXELS * 2),
                imagery_data: Vec::with_capacity(TILE_SIZE_PIXELS * TILE_SIZE_PIXELS * 3),
            };

            let elev_raster = elevation_band.read_as::<i16>((elev_x as isize, elev_y as isize), (elev_size_x, elev_size_y), (elev_size_x, elev_size_y)).expect("Unable to read raster area from elevation band");
            let red_raster = red_band.read_as::<u8>((img_x as isize, img_y as isize), (img_size_x, img_size_y), (img_size_x, img_size_y)).expect("Unable to read raster area from red band");
            let green_raster = green_band.read_as::<u8>((img_x as isize, img_y as isize), (img_size_x, img_size_y), (img_size_x, img_size_y)).expect("Unable to read raster area from green band");
            let blue_raster = blue_band.read_as::<u8>((img_x as isize, img_y as isize), (img_size_x, img_size_y), (img_size_x, img_size_y)).expect("Unable to read raster area from blue band");

            for pixel_y in 0..TILE_SIZE_PIXELS {
                let pixel_percent_y = (pixel_y as f64) / ((TILE_SIZE_PIXELS - 1) as f64);
                let elev_raster_y = (((elev_size_y - 1) as f64) * pixel_percent_y) as usize;
                let img_raster_y = (((img_size_y - 1) as f64) * pixel_percent_y) as usize;

                for pixel_x in 0..TILE_SIZE_PIXELS {
                    let pixel_percent_x = (pixel_x as f64) / ((TILE_SIZE_PIXELS - 1) as f64);
                    let elev_raster_x = (((elev_size_x - 1) as f64) * pixel_percent_x) as usize;
                    let img_raster_x = (((img_size_x - 1) as f64) * pixel_percent_x) as usize;

                    let elev_index = elev_raster_y * elev_size_x + elev_raster_x;
                    let img_index = img_raster_y * img_size_x + img_raster_x;
                    let elev = elev_raster.data[elev_index];
                    let red = red_raster.data[img_index];
                    let green = green_raster.data[img_index];
                    let blue = blue_raster.data[img_index];
                    tile.elevation_data.extend_from_slice(&elev.to_be_bytes());
                    tile.imagery_data.push(red);
                    tile.imagery_data.push(green);
                    tile.imagery_data.push(blue);
                }
            }

            batch.push(tile);

            if batch.len() >= BATCH_SIZE {
                diesel::insert_into(tiles::table)
                    .values(&batch)
                    .execute(&connection)
                    .expect("Unable to insert tile");

                batch.clear();
            }
        }

        if !batch.is_empty() {
            diesel::insert_into(tiles::table)
                .values(&batch)
                .execute(&connection)
                .expect("Unable to insert tile");

            batch.clear();
        }

        let mut data = progress.lock().unwrap();
        *data += 1;
        let prog = (*data as f64) / (NUM_TILES_LAT as f64) * 100.0;
        println!("Row {} finished, {}% complete", tile_y, prog);
    }
}

fn main() {
    let num_threads = num_cpus::get();
    let progress = Arc::new(Mutex::new(0));
    
    (0..num_threads).into_par_iter().for_each(|i| create_tiles(i, num_threads, Arc::clone(&progress)));
}
