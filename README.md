# Background
Realworld is ["The Mother of all demo apps"](https://codebase.show/projects/realworld).  
This project specifies a precise API description. Thus there are a large number of independent front-end implementations and back-end implementations. Any front-end can working with another set of back-ends.  
On the other hand, Realworld defines a full-featured forum system. In order to achieve this exquisite small goal, a lot of practical technology needs to be applied. If the front-end and back-end solutions achieve this goal, and it is easy to verify the development potential in other Internet application topics.  

# Rust+Warp
This back-end implementation uses Rust language and Warp Web framework. The database system uses PostgreSQL through Diesel ORM.  
Most of the program code comes from another Rust RealWorld implementation: [Rust + Rocket](https://github.com/TatriX/realworld-rust-rocket).  
I used to be a user of Rocket and Actix, but neither of them satisfied me. The former uses the Nightly version of Rust. It is also unstable when working, process lockup often. The latter has a large amount of memory usage for unknown reasons.  
Finally, I moved to the Warp framework, at least until now, Warp's performance is very satisfying.  

# Difference with official specs
* This implementation embed static file server, you can directly put front-end in to `./dist/` folder
* Add upload function for some file exchange request 
* Addition command line tool

# Install
* Setup PostgreSQL database
* Install PostgreSQL dev library (Include PostgeSQL in Linux version)  

mac: 
```bash
brew install libpq
```
ubuntu:   
```bash 
sudo apt install libpq-dev postgresql-server-dev-all
```

* Run init.sql to create user and database in psql client:  
```bash
sudo -u postgres psql -f init.sql 
```
```sql
CREATE USER realworld WITH PASSWORD 'realworld';
CREATE DATABASE realworld;
GRANT ALL PRIVILEGES ON DATABASE realworld TO realworld;
```
* setting `.env` configure file in working root directory as sample below:
```bash
DATABASE_URL=postgres://realworld:realworld@127.0.0.1:6551/realworld
SECRET_KEY="8Xui8SN4mI+7egV/9dlfYYLGQJeEx4+DwmSQLwDVXJg="
WEB_URL="0.0.0.0:8000"
SERVER_NAME="RealWorld Server"
#RUST_LOG=info
#RUST_LOG=trace
RUST_LOG=debug
PUBLIC_BOARD=true
```
`.env.txt` file content is same as up, you directly rename as `.env` then edit as your wish

* Run diesel setup database tables
```bash
# install diesel client
cargo install diesel_cli --no-default-features --features "postgres"
# setting up tables
diesel migration run
```
* Copy Realworld front-end to `./dist/` folder, currently I put Vue version client here

# run in debug mode
```bash
cargo run --bin realworld-warp
```

# compile
```bash
cargo build --release
```

# run
There will be 2 binary compiled:  
* `realworld-warp` is main program
* `realworld-cli` is a command line tool help list user/article/follow, or delete them  

You need run in the root path of project, because `.env`  `./dist/` and some other dependency path location
```bash
./target/release/realworld-warp
```

# Demo address
<http://39.105.37.153:8000>  
(This is not a permenant server, so may not exist long)


# other
Upload function and Static file holder need front-end cooperate, suggest use [my vue version](https://github.com/formoon/vue-realworld-example-app), it's a revisionary from [Vue2 version RealWorld Frontend](https://github.com/gothinkster/vue-realworld-example-app)


