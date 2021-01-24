use rocksdb::{DB, Options};

pub fn establish_connection(db_path: &str) -> Result<DB, rocksdb::Error> {
    let mut db_opts = Options::default();
    db_opts.create_if_missing(true);
    DB::open(&db_opts, db_path)
}