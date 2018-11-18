process.env.PGUSER = "postgres";
process.env.PGHOST = 'localhost'
process.env.PGPASSWORD = '123'
process.env.PGDATABASE = 'postgres'
process.env.PGPORT = '5432'


const cluster = require('cluster');

// if (cluster.isMaster) {

//   for (let i = 0; i < 8; i++) {
//     cluster.fork();
//   }
// } else {

const { Pool, Client } = require('pg')
const client = new Pool({
  max: 10
})

const http = require('http');

async function exec() {
  await client.connect();
}

exec();

http.createServer(async (req, res) => {
  await client.query("INSERT INTO person (name, data) VALUES ('Steven', '')");
  res.writeHead(200)
  res.end();
}).listen(3000, () => {
  console.log("Server is running");
});

// }