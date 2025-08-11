use std::fs;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;

use ic_cdk::export_candid;

use ic_rusqlite::close_connection;
use ic_rusqlite::get_db_path;
use ic_rusqlite::types::Type;
use ic_rusqlite::with_connection;

#[ic_cdk::update]
fn query(sql: String) -> Vec<Vec<Option<String>>> {
    let start = ic_instruction_counter();

    let res = with_connection(|conn| {
        let mut stmt = conn.prepare(&sql).unwrap();
        let cnt = stmt.column_count();

        // create rows iterator
        let rows_iter = stmt
            .query_map([], |row| {
                let mut vec: Vec<Option<String>> = Vec::new();

                for idx in 0..cnt {
                    let v = row.get_ref_unwrap(idx);
                    match v.data_type() {
                        Type::Null => vec.push(None),
                        Type::Integer => vec.push(Some(v.as_i64().unwrap().to_string())),
                        Type::Real => vec.push(Some(v.as_f64().unwrap().to_string())),
                        Type::Text => vec.push(Some(v.as_str().unwrap().parse().unwrap())),
                        Type::Blob => vec.push(Some(hex::encode(v.as_blob().unwrap()))),
                    }
                }

                Ok(vec)
            })
            .unwrap();

        let res: Vec<Vec<Option<String>>> = rows_iter.filter_map(Result::ok).collect();
        res
    });

    let end = ic_instruction_counter();

    println!("Query {}\n execution time: {}", sql, end - start);

    res
}

#[ic_cdk::update]
pub fn upload_database(db: Vec<u8>) {
    close_connection();

    let mut file = OpenOptions::new()
        .write(true)
        .create(false)
        .truncate(true)
        .open(get_db_path())
        .unwrap();

    file.write_all(&db).unwrap();
}

#[ic_cdk::query]
pub fn download_database() -> Vec<u8> {
    close_connection();

    let mut file = OpenOptions::new()
        .read(true)
        .open(get_db_path())
        .expect("Database file not found!");

    let mut buf = Vec::new();
    file.read_to_end(&mut buf).unwrap();

    buf
}

#[ic_cdk::update]
pub fn close_database() {
    close_connection();
}

#[ic_cdk::update]
pub fn get_db_size() -> u64 {
    close_connection();

    // Get file metadata

    fs::metadata(get_db_path()).unwrap().len()
}

#[ic_cdk::query]
pub fn first_bytes() -> String {
    close_connection();

    let mut file = OpenOptions::new()
        .read(true)
        .open(get_db_path())
        .expect("Database file not found!");

    let mut buf = vec![0u8; 100];
    let bytes_read = file.read(&mut buf).unwrap();
    buf.truncate(bytes_read);

    hex::encode(&buf)
}

fn execute(sql: &str) {
    with_connection(|conn| {
        conn.execute(sql, ()).unwrap();
    })
}

#[ic_cdk::update]
pub fn execute_batch(sql: &str) {
    with_connection(|conn| {
        conn.execute_batch(sql).unwrap();
    })
}

fn create_tables() {
    execute(
        "
            CREATE TABLE IF NOT EXISTS users (
                        user_id INTEGER PRIMARY KEY AUTOINCREMENT,
                        username TEXT NOT NULL,
                        email TEXT NOT NULL,
                        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                    )
        ",
    );

    execute(
        "
            CREATE TABLE IF NOT EXISTS orders (
                order_id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                amount REAL NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (user_id) REFERENCES users(user_id)
            )
        ",
    );
}

fn create_indices() {
    execute("CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);");
    execute("CREATE INDEX IF NOT EXISTS idx_orders_user_id ON orders(user_id);");
}

#[ic_cdk::update]
fn create_chinook_indices() {
    execute("CREATE INDEX IF NOT EXISTS idx_customers_first_name ON customers(firstname);");
    execute("CREATE INDEX IF NOT EXISTS idx_customers_last_name ON customers(lastname);");
}

#[ic_cdk::init]
fn init() {
    // create some default database
    create_tables();
    create_indices();
}

#[ic_cdk::update]
fn get_tables() -> Vec<String> {
    with_connection(|conn| {
        let mut stmt = conn
            .prepare(
                r"SELECT 
                    name
                FROM 
                    sqlite_schema
                WHERE 
                    type ='table' AND 
                    name NOT LIKE 'sqlite_%';",
            )
            .unwrap();

        let table_names = stmt
            .query_map([], |row| {
                let name: String = row.get(0).unwrap();

                Ok(name)
            })
            .unwrap();

        let res: Vec<String> = table_names.filter_map(Result::ok).collect();
        res
    })
}

use ic_cdk::api::instruction_counter as ic_instruction_counter;

#[ic_cdk::update]
fn add_customers(offset: u64) -> u64 {
    let start = ic_instruction_counter();

    with_connection(|mut conn| {
        let tx = conn.transaction().unwrap();

        let sql = String::from(
            "insert into customers (firstname, lastname, email, address, city, state, country, postalcode, phone, fax) values (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        );

        {
            let mut stmt = tx.prepare_cached(&sql).unwrap();

            let mut i = 0;

            while i < 100000000 {
                let id = offset + i + 1;
                let name = format!("{id}customer_name{id}");
                let last_name = format!(
                    "{}customer_last_name{}",
                    (id * 120301070105014129u64) % u64::MAX,
                    (id * 120301070105014129u64) % u64::MAX
                );
                let email = format!("{id}customer@example.com");
                let dummy = "is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book. It has survived not only five centuries, but also the leap into electronic typesetting, remaining essentially unchanged. It was popularised in the 1960s with the release of Letraset sheets containing Lorem Ipsum passages, and more recently with desktop publishing software like Aldus PageMaker including versions of Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book. It has survived not only five centuries, but also the leap into electronic typesetting, remaining essentially unchanged. It was popularised in the 1960s with the release of Letraset sheets containing Lorem Ipsum passages, and more recently with desktop publishing software like Aldus PageMaker including versions of Lorem Ipsumis simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book. It has survived not only five centuries, but also the leap into electronic typesetting, remaining essentially unchanged. It was popularised in the 1960s with the release of Letraset sheets containing Lorem Ipsum passages, and more recently with desktop publishing software like Aldus PageMaker including versions of Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book. It has survived not only five centuries, but also the leap into electronic typesetting, remaining essentially unchanged. It was popularised in the 1960s with the release of Letraset sheets containing Lorem Ipsum passages, and more recently with desktop publishing software like Aldus PageMaker including versions of Lorem Ipsumis simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book. It has survived not only five centuries, but also the leap into electronic typesetting, remaining essentially unchanged. It was popularised in the 1960s with the release of Letraset sheets containing Lorem Ipsum passages, and more recently with desktop publishing software like Aldus PageMaker including versions of Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book. It has survived not only five centuries, but also the leap into electronic typesetting, remaining essentially unchanged. It was popularised in the 1960s with the release of Letraset sheets containing Lorem Ipsum passages, and more recently with desktop publishing software like Aldus PageMaker including versions of Lorem Ipsumis simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book. It has survived not only five centuries, but also the leap into electronic typesetting, remaining essentially unchanged. It was popularised in the 1960s with the release of Letraset sheets containing Lorem Ipsum passages, and more recently with desktop publishing software like Aldus PageMaker including versions of Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book. It has survived not only five centuries, but also the leap into electronic typesetting, remaining essentially unchanged. It was popularised in the 1960s with the release of Letraset sheets containing Lorem Ipsum passages, and more recently with desktop publishing software like Aldus PageMaker including versions of Lorem Ipsumis simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book. It has survived not only five centuries, but also the leap into electronic typesetting, remaining essentially unchanged. It was popularised in the 1960s with the release of Letraset sheets containing Lorem Ipsum passages, and more recently with desktop publishing software like Aldus PageMaker including versions of Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book. It has survived not only five centuries, but also the leap into electronic typesetting, remaining essentially unchanged. It was popularised in the 1960s with the release of Letraset sheets containing Lorem Ipsum passages, and more recently with desktop publishing software like Aldus PageMaker including versions of Lorem Ipsumis simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book. It has survived not only five centuries, but also the leap into electronic typesetting, remaining essentially unchanged. It was popularised in the 1960s with the release of Letraset sheets containing Lorem Ipsum passages, and more recently with desktop publishing software like Aldus PageMaker including versions of Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book. It has survived not only five centuries, but also the leap into electronic typesetting, remaining essentially unchanged. It was popularised in the 1960s with the release of Letraset sheets containing Lorem Ipsum passages, and more recently with desktop publishing software like Aldus PageMaker including versions of Lorem Ipsumis simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book. It has survived not only five centuries, but also the leap into electronic typesetting, remaining essentially unchanged. It was popularised in the 1960s with the release of Letraset sheets containing Lorem Ipsum passages, and more recently with desktop publishing software like Aldus PageMaker including versions of Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book. It has survived not only five centuries, but also the leap into electronic typesetting, remaining essentially unchanged. It was popularised in the 1960s with the release of Letraset sheets containing Lorem Ipsum passages, and more recently with desktop publishing software like Aldus PageMaker including versions of Lorem Ipsumis simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book. It has survived not only five centuries, but also the leap into electronic typesetting, remaining essentially unchanged. It was popularised in the 1960s with the release of Letraset sheets containing Lorem Ipsum passages, and more recently with desktop publishing software like Aldus PageMaker including versions of Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book. It has survived not only five centuries, but also the leap into electronic typesetting, remaining essentially unchanged. It was popularised in the 1960s with the release of Letraset sheets containing Lorem Ipsum passages, and more recently with desktop publishing software like Aldus PageMaker including versions of Lorem Ipsumis simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book. It has survived not only five centuries, but also the leap into electronic typesetting, remaining essentially unchanged. It was popularised in the 1960s with the release of Letraset sheets containing Lorem Ipsum passages, and more recently with desktop publishing software like Aldus PageMaker including versions of Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book. It has survived not only five centuries, but also the leap into electronic typesetting, remaining essentially unchanged. It was popularised in the 1960s with the release of Letraset sheets containing Lorem Ipsum passages, and more recently with desktop publishing software like Aldus PageMaker including versions of Lorem Ipsum";

                stmt.execute(ic_rusqlite::params![
                    name, last_name, email, dummy, dummy, dummy, dummy, dummy, dummy, dummy
                ])
                .expect("insert of a user failed!");

                i += 1;

                //
                let end = ic_instruction_counter();
                if end - start > 20000000000 {
                    break;
                }
            }
        }

        tx.commit().expect("COMMIT USER INSERTION FAILED!");
    });

    let end = ic_instruction_counter();

    end - start
}

/*
#[ic_cdk::update]
fn add_users(offset: usize, count: usize) -> Result {
    with_connection(|mut conn| {
        let tx = conn.transaction().unwrap();

        let sql = String::from("insert into users (username, email) values (?, ?)");

        {
            let mut stmt = tx.prepare_cached(&sql).unwrap();

            let mut i = 0;

            while i < count {
                let id = offset + i + 1;
                let username = format!("user{id}");
                let email = format!("user{id}@example.com");

                stmt.execute(ic_rusqlite::params![username, email])
                    .expect("insert of a user failed!");

                i += 1;
            }
        }

        tx.commit().expect("COMMIT USER INSERTION FAILED!");

        Ok(String::from("bench1_insert_person OK"))
    })
}

#[ic_cdk::update]
fn add_orders(offset: usize, count: usize, id_mod: usize) -> Result {
    with_connection(|mut conn| {
        let tx = conn.transaction().unwrap();

        let sql = String::from("insert into orders (user_id, amount) values (?, ?)");

        {
            let mut stmt = tx.prepare_cached(&sql).unwrap();

            let mut i = 0;

            while i < count {
                let id = (offset + i + 1) * 13 % id_mod + 1;

                stmt.execute(ic_rusqlite::params![id, (id * 100 + id * 17) / 15])
                    .unwrap_or_else(|_| {
                        panic!(
                            "insertion of a new order failed: i = {i} count = {count} id = {id}!"
                        )
                    });

                i += 1;
            }
        }

        tx.commit().expect("COMMIT ORDER INSERTION FAILED!");

        Ok("add_orders OK".to_string())
    })
}

*/

export_candid!();
