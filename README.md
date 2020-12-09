# About
This repository contains two programs. One builds a database of Mars elevation and orthoimagery data from two freely available datasets. The other runs a HTTP server that accepts web requests to make queries to the database and returns the results. There is a [3D visualization](https://github.com/KSaunders98/mars_visualization) that makes use of this database.
# Building
## Dependencies
In order to build this program, some dependencies and setup steps are needed:
- Download the required datasets from [here](https://astrogeology.usgs.gov/search/map/Mars/Viking/MDIM21/Mars_Viking_MDIM21_ClrMosaic_global_232m) and [here](https://astrogeology.usgs.gov/search/map/Mars/Topography/HRSC_MOLA_Blend/Mars_HRSC_MOLA_BlendDEM_Global_200mp) into the datasets directory in this project. Do not rename them.
- Ensure MySQL server is installed on your machine.
- Create a file ".env" in the project root with the contents `DATABASE_URL=<your MySQL database url here>`.
    - NOTE: The database does not need to exist. If the database given does not exist, it will be created, so pick a name that makes sense.
- Ensure the MySQL client development package is installed on your machine (`sudo apt-get install libmysqlclient-dev` on Ubuntu)
- Run `cargo install diesel_cli --no-default-features --features "mysql"`
    - NOTE: If the mysqlclient shared library isn't in your system path (likely if you're using Windows), you will need to create an environment variable called `MYSQLCLIENT_LIB_DIR` that points to the directory where it is installed for this command to succeed.
- Run `diesel setup` to create the database and schema.
- Ensure GDAL development libraries are installed on your machine (See [here](https://mothergeo-py.readthedocs.io/en/latest/development/how-to/gdal-ubuntu-pkg.html) for details on how to install them on Ubuntu).
- Create an environment variable called `GDAL_LIB_DIR` that points to the directory where libgdal.so is located.
    - NOTE: This environment variable only needs to be present for the first `cargo build` or `cargo run`, then it is no longer necessary (unless you run `cargo clean`).
## Final Build Steps
- Run `cargo build --release` to download and compile the rest of the dependencies and build the programs.
- Run `cargo run --release -p init` to run the program that builds the database. Depending on your storage medium, this may take a while. On my machine with the default MySQL database engine + settings, this took around 45GB of space.
- Run `cargo run --release` to run the server program on `*.*.*.*:3000`. At this point, the [visualization program](https://github.com/KSaunders98/mars_visualization) can be run. Press ctrl+c to stop the server when you are done.