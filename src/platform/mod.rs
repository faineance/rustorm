pub mod postgres;

pub use self::postgres::Postgres;

use database::Database;




pub enum Platform{
    Postgres(Postgres),
    Sqlite,
    Oracle,
    Mysql,
}

impl Platform{
    
    pub fn as_ref(&self)->&Database{
        match *self{
            Platform::Postgres(ref pg) => pg,
            _ => panic!("others not yet..")
        }
    }
    
}

impl Drop for Platform {
    fn drop(&mut self) {
        println!("Warning: Dropping a connection is expensive, please return this to the pool");
    }
}
