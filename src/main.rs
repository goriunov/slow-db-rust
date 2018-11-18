extern crate futures;
extern crate postgres;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate tokio;
extern crate tokio_threadpool;
extern crate warp;

use futures::prelude::*;
use warp::Filter;

use r2d2_postgres::{PostgresConnectionManager, TlsMode};

const PORT: u16 = 3000;
const DB_URL: &'static str = "postgres://path";

type PgPool = r2d2::Pool<PostgresConnectionManager>;
type PooledPg = r2d2::PooledConnection<PostgresConnectionManager>;

fn db_pool() -> PgPool {
  let manager = PostgresConnectionManager::new(DB_URL, TlsMode::None).unwrap();
  r2d2::Pool::new(manager).unwrap() // need to specify number of connections default 10
}

struct Person {
  id: i32,
  name: String,
  data: Option<Vec<u8>>,
}

fn main() {
  let pool = db_pool();

  let db = warp::any()
    .map(move || pool.clone())
    .and_then(|pg: PgPool| match pg.get() {
      Ok(conn) => Ok(conn),
      Err(e) => Err(warp::reject::server_error()),
    });

  let index_load = warp::path::end().and(db).and_then(|db: PooledPg| {
    futures::future::poll_fn(move || {
      tokio_threadpool::blocking(|| {
        let me = Person {
          id: 0,
          name: "Steven".to_string(),
          data: None,
        };

        let call = db
          .prepare_cached("INSERT INTO person (name, data) VALUES ($1, $2)")
          .unwrap();

        //  db
        //         .execute(
        //           "CREATE TABLE person (
        //                   id              SERIAL PRIMARY KEY,
        //                   name            VARCHAR NOT NULL,
        //                   data            BYTEA
        //                 )",
        //           &[],
        //         )
        //         .unwrap();
        call.execute(&[&me.name, &me.data]).unwrap();
      })
    })
    .map(|_| Ok("Completed req"))
    .map_err(|_| warp::reject())
  });

  println!("Server is running on port {}", PORT);
  let server = warp::serve(warp::get2().and(index_load)).bind(([0, 0, 0, 0], PORT));

  let mut thread_pool = tokio_threadpool::Builder::new();

  thread_pool
    .pool_size(4)
    .max_blocking(1000)
    .name_prefix("helllo-world-");

  let mut runtime = tokio::runtime::Builder::new()
    .threadpool_builder(thread_pool)
    .build()
    .unwrap();

  runtime.spawn(server);

  runtime.shutdown_on_idle().wait().unwrap();
}
