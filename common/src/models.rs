use super::schema::tiles;

#[derive(Insertable, Queryable)]
#[table_name="tiles"]
pub struct Tile {
    pub id: u32,
    pub latitude: i32,
    pub longitude: i32,
    pub elevation_data: Vec<u8>,
    pub imagery_data: Vec<u8>,
}