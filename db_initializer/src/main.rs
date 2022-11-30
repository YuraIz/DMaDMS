// use std::vec;

use openssl::ssl::{SslConnector, SslMethod};
use postgres::Client;
use postgres_openssl::MakeTlsConnector;

fn connect() -> Client {
    let postgres_host = std::env::var("POSTGRES_HOST").unwrap();
    let postgres_user = std::env::var("POSTGRES_USER").unwrap();
    let postgres_password = std::env::var("POSTGRES_PASSWORD").unwrap();
    let postgres_dbname = std::env::var("POSTGRES_DBNAME").unwrap();

    let ssl_connector = SslConnector::builder(SslMethod::tls()).unwrap().build();

    let connector = MakeTlsConnector::new(ssl_connector);

    let params = format!("host={postgres_host} user={postgres_user} password={postgres_password} dbname={postgres_dbname}");

    Client::connect(&params, connector).expect("can't connect to postgresql database")
}

fn drop_tables(client: &mut Client) {
    for table_name in [
        "countries",
        "suppliers",
        "product_categories",
        "product_subcategories",
        "products",
        "clients",
        "client_addresses",
        "product_requirements",
        "warehouses",
        "product_locations",
        "user_roles",
        "users",
    ] {
        _ = client.batch_execute(&format!("DROP TABLE {table_name} CASCADE"));
    }
}

fn create_tables(client: &mut Client) -> Result<(), postgres::Error> {
    client.batch_execute("BEGIN TRANSACTION")?;

    client.batch_execute(
        "
        CREATE TABLE countries (
            country_id  SERIAL PRIMARY KEY,
            name        TEXT UNIQUE NOT NULL
        )
    ",
    )?;

    client.batch_execute(
        "
        CREATE TABLE suppliers (
            supplier_id SERIAL PRIMARY KEY,
            country_id  INTEGER REFERENCES countries NOT NULL,
            name        TEXT NOT NULL,
            email       TEXT NOT NULL
        )
    ",
    )?;

    client.batch_execute(
        "
        CREATE TABLE product_categories (
            category_id SERIAL PRIMARY KEY,
            name        TEXT UNIQUE NOT NULL
        )
    ",
    )?;

    client.batch_execute(
        "
        CREATE TABLE product_subcategories (
            subcategory_id  SERIAL PRIMARY KEY,
            category_id     INTEGER REFERENCES product_categories NOT NULL,
            name            TEXT UNIQUE NOT NULL
        )
    ",
    )?;

    client.batch_execute(
        "
        CREATE TABLE products (
            product_id      SERIAL PRIMARY KEY,
            supplier_id     INTEGER REFERENCES suppliers NOT NULL,
            subcategory_id  INTEGER REFERENCES product_subcategories NOT NULL,
            name            TEXT UNIQUE NOT NULL
        )
    ",
    )?;

    client.batch_execute(
        "
        CREATE TABLE clients (
            client_id   SERIAL PRIMARY KEY,
            name        TEXT UNIQUE NOT NULL,
            email       TEXT NOT NULL
        )
    ",
    )?;

    client.batch_execute(
        "
        CREATE TABLE client_addresses (
            client_address_id   SERIAL PRIMARY KEY,
            client_id           INTEGER REFERENCES clients NOT NULL,
            address             TEXT NOT NULL
        )
    ",
    )?;

    client.batch_execute(
        "
        CREATE TABLE product_requirements (
            product_requirement_id  SERIAL PRIMARY KEY,
            product_id              INTEGER REFERENCES products NOT NULL,
            client_address_id       INTEGER REFERENCES client_addresses NOT NULL,
            count                   INTEGER NOT NULL,
            CHECK (count >= 0)
        )
    ",
    )?;

    client.batch_execute(
        "
        CREATE TABLE warehouses (
            warehouse_id    SERIAL PRIMARY KEY,
            address         TEXT UNIQUE NOT NULL
        )
        ",
    )?;

    client.batch_execute(
        "
        CREATE TABLE product_locations (
            product_location_id SERIAL PRIMARY KEY,
            warehouse_id        INTEGER REFERENCES warehouses NOT NULL,
            product_id          INTEGER REFERENCES products NOT NULL,
            count               INTEGER NOT NULL,
            CHECK (count >= 0)
        )
    ",
    )?;

    client.batch_execute(
        "
        CREATE TABLE user_roles (
            user_role_id    SERIAL PRIMARY KEY,
            name            TEXT UNIQUE NOT NULL
        )
    ",
    )?;

    client.batch_execute("CREATE EXTENSION IF NOT EXISTS pgcrypto")?;

    client.batch_execute(
        "
        CREATE TABLE users (
            user_id             SERIAL PRIMARY KEY,
            supplier_id         INTEGER REFERENCES suppliers UNIQUE, -- NULLABLE
            client_id           INTEGER REFERENCES clients UNIQUE, -- NULLABLE
            user_role_id        INTEGER REFERENCES user_roles NOT NULL,
            name                TEXT UNIQUE NOT NULL,
            password            TEXT NOT NULL, -- use encryption
            CHECK ((supplier_id IS NULL) OR (client_id IS NULL))
        )
    ",
    )?;

    client.batch_execute("COMMIT TRANSACTION")?;

    Ok(())
}

fn create_indexes(client: &mut Client) -> Result<(), postgres::Error> {
    client.batch_execute(
        "
        CREATE INDEX user_index
        ON users(supplier_id, client_id)
    ",
    )?;

    client.batch_execute(
        "
        CREATE INDEX user_role_index
        ON user_roles(name)
    ",
    )?;

    Ok(())
}

fn init_tables(client: &mut Client) -> Result<(), postgres::Error> {
    /* TODO LIST:
     * client_addresses
     * clients
     * countries
     * product_categories
     * product_locations
     * product_requirements
     * product_subcategories
     * products
     * suppliers
     * user
     * user_roles
     * warehouses
     */

    client.batch_execute("BEGIN TRANSACTION")?;

    let countries = include!("init_data/countries");

    for country in countries {
        client.execute(
            "INSERT INTO countries (name) VALUES ($1)",
            &[&country.to_owned()],
        )?;
    }

    let supplier_names = include!("init_data/suppliers").iter();
    let emails = include!("init_data/emails").iter();

    let mut suppliers = supplier_names.zip(emails);

    let country_ids: Vec<i32> = client
        .query("SELECT country_id FROM countries", &[])?
        .iter()
        .map(|row| row.get("country_id"))
        .collect();

    for country_id in country_ids.iter().cycle() {
        if let Some(supplier) = suppliers.next() {
            client.execute(
                "
                INSERT INTO suppliers (country_id, name, email)
                VALUES ($1, $2, $3)
                ",
                &[&country_id, supplier.0, supplier.1],
            )?;
        } else {
            break;
        }
    }

    let client_names = include!("init_data/clients").iter();
    let emails = include!("init_data/emails").iter();
    let clients = client_names.zip(emails);

    for my_client in clients {
        client.execute(
            "
            INSERT INTO clients (name, email)
            VALUES ($1, $2)
            ",
            &[my_client.0, my_client.1],
        )?;
    }

    let product_categories = include!("init_data/product_categories");

    for (category, subcategories) in product_categories {
        if let &[ref row] = &client.query(
            "
            INSERT INTO product_categories (name)
            VALUES ($1)
            RETURNING (category_id)
            ",
            &[&category],
        )?[..]
        {
            let category_id: i32 = row.get("category_id");

            for subcategory in subcategories {
                client.execute(
                    "
                    INSERT INTO product_subcategories (category_id, name)
                    VALUES ($1, $2)
                    ",
                    &[&category_id, &subcategory],
                )?;
            }
        }
    }

    let subcategory_ids: Vec<i32> = client
        .query("SELECT subcategory_id FROM product_subcategories", &[])?
        .iter()
        .map(|row| row.get("subcategory_id"))
        .collect();

    let supplier_ids: Vec<i32> = client
        .query("SELECT supplier_id FROM suppliers", &[])?
        .iter()
        .map(|row| row.get("supplier_id"))
        .collect();

    let product_names = include!("init_data/products");

    let products = product_names.iter().zip(
        subcategory_ids
            .iter()
            .cycle()
            .zip(supplier_ids.iter().cycle()),
    );

    for (product_name, (subcategory_id, supplier_id)) in products {
        client.execute(
            "
            INSERT INTO products (supplier_id, subcategory_id, name)
            VALUES ($1, $2, $3)
            ",
            &[&supplier_id, &subcategory_id, &product_name],
        )?;
    }

    let addresses = include!("init_data/addresses");

    for address in &addresses {
        client.execute(
            "
        INSERT INTO warehouses (address)
        VALUES ($1)
        ",
            &[address],
        )?;
    }

    let client_ids: Vec<i32> = client
        .query("SELECT client_id FROM clients", &[])?
        .iter()
        .map(|row| row.get("client_id"))
        .collect();

    let client_addresses = addresses.iter().zip(client_ids.iter().cycle());

    for (address, client_id) in client_addresses {
        client.execute(
            "
            INSERT INTO client_addresses (client_id, address)
            VALUES ($1, $2)
            ",
            &[&client_id, address],
        )?;
    }

    let client_address_ids: Vec<i32> = client
        .query("SELECT client_address_id from client_addresses", &[])?
        .iter()
        .map(|row| row.get("client_address_id"))
        .collect();

    let product_ids: Vec<i32> = client
        .query("SELECT product_id from products", &[])?
        .iter()
        .map(|row| row.get("product_id"))
        .take(10)
        .collect();

    for client_address_id in client_address_ids {
        for product_id in &product_ids {
            let count = client_address_id
                .wrapping_mul(73)
                .wrapping_add(product_id.wrapping_add(42))
                % 300;

            if count != 0 {
                client.execute(
                    "
                    INSERT INTO product_requirements (client_address_id, product_id, count)
                    VALUES ($1, $2, $3)
                    ",
                    &[&client_address_id, &product_id, &count],
                )?;
            }
        }
    }

    let warehouse_ids: Vec<i32> = client
        .query("SELECT warehouse_id from warehouses", &[])?
        .iter()
        .map(|row| row.get("warehouse_id"))
        .collect();

    for warehouse_id in warehouse_ids {
        for product_id in &product_ids {
            let count = warehouse_id
                .wrapping_mul(73)
                .wrapping_add(product_id.wrapping_add(42))
                % 300;

            if count != 0 {
                client.execute(
                    "
                    INSERT INTO product_locations (warehouse_id, product_id, count)
                    VALUES ($1, $2, $3)
                    ",
                    &[&warehouse_id, &product_id, &count],
                )?;
            }
        }
    }

    let user_roles = ["admin", "manager", "client", "supplier"];

    for user_role in user_roles {
        client.execute(
            "
            INSERT INTO user_roles (name)
            VALUES ($1)
        ",
            &[&user_role],
        )?;
    }

    client.batch_execute(
        "
        INSERT INTO users (name, user_role_id, password) values (
            'Gigachad', 
            (
                SELECT user_role_id
                FROM user_roles
                WHERE name = 'admin'
            ),
            crypt('adminadmin', gen_salt('md5'))
    )
    ",
    )?;

    let managers = [
        ("Helmer", "array"),
        ("Macey", "capacitor"),
        ("Melvina", "interface"),
        ("Priscilla", "driver"),
        ("Mollie", "capacitor"),
        ("Jaren", "driver"),
        ("Addison", "port"),
        ("Jerrold", "firewall"),
    ];

    let manager_role_id: i32 = client
        .query_one(
            "
                SELECT user_role_id
                FROM user_roles
                WHERE name = 'manager'
            ",
            &[],
        )?
        .get("user_role_id");

    for manager in managers {
        client.execute(
            "
        INSERT INTO users (name, password, user_role_id)
        VALUES ($1, 
            crypt($2, gen_salt('md5')), $3)",
            &[&manager.0, &manager.1, &manager_role_id],
        )?;
    }

    let supplier_role_id: i32 = client
        .query_one(
            "
                SELECT user_role_id
                FROM user_roles
                WHERE name = 'supplier'
            ",
            &[],
        )?
        .get("user_role_id");

    let supplier_ids: Vec<i32> = client
        .query("SELECT supplier_id FROM suppliers", &[])?
        .iter()
        .map(|row| row.get("supplier_id"))
        .collect();

    for supplier_id in supplier_ids {
        client.execute(
            "
            INSERT INTO users (name, password, user_role_id, supplier_id)
            VALUES (
                (
                    SELECT name
                    FROM suppliers
                    WHERE supplier_id = $1
                ),
                crypt('password', gen_salt('md5')),
                $2,
                $1
            )
            ",
            &[&supplier_id, &supplier_role_id],
        )?;
    }

    let client_role_id: i32 = client
        .query_one(
            "
                SELECT user_role_id
                FROM user_roles
                WHERE name = 'client'
            ",
            &[],
        )?
        .get("user_role_id");

    let client_ids: Vec<i32> = client
        .query("SELECT client_id FROM clients", &[])?
        .iter()
        .map(|row| row.get("client_id"))
        .collect();

    for client_id in client_ids {
        client.execute(
            "
            INSERT INTO users (name, password, user_role_id, client_id)
            VALUES (
                (
                    SELECT name
                    FROM clients
                    WHERE client_id = $1
                ),
                crypt('password', gen_salt('md5')),
                $2,
                $1
            )
            ",
            &[&client_id, &client_role_id],
        )?;
    }

    client.batch_execute("COMMIT TRANSACTION")?;

    Ok(())
}

/*
"
    SELECT product_categories.name as category, product_subcategories.name as subcategory
    FROM product_categories
    CROSS JOIN product_subcategories
"
*/

fn demo_queries(client: &mut Client) -> Result<(), postgres::Error> {
    /* TODO LIST:
     * client_addresses
     * clients
     * countries +
     * product_categories +
     * product_locations
     * product_requirements
     * product_subcategories +
     * products
     * suppliers
     * user
     * user_roles
     * warehouses
     */

    // List of suppliers with name, email and country. +
    // List of products for category or subcategory. +
    // List of product requierments for each client address.
    // List of awailable products per warehouse.
    // List of countries, categories and subcategories.
    // List of users.
    // List of all addresses for client.

    let countries: Vec<String> = client
        .query(
            "
                SELECT name FROM countries
            ",
            &[],
        )?
        .iter()
        .map(|row| row.get("name"))
        .collect();

    println!(
        "
Countries
"
    );

    for country in countries {
        println!("{country}")
    }

    let caterories: Vec<(String, String)> = client
        .query(
            "
                SELECT product_categories.name as category, product_subcategories.name as subcategory
                FROM product_categories
                INNER JOIN product_subcategories 
                ON product_categories.category_id = product_subcategories.category_id;
            ",
            &[],
        )?
        .iter()
        .map(|row| (row.get("category"), row.get("subcategory")))
        .collect();

    println!(
        "
Subcategories per each category
Categories                     Subcategories
"
    );

    for (category, subcategory) in caterories {
        println!("{category:30} {subcategory}")
    }

    let suppliers: Vec<(String, String, String)> = client
        .query(
            "
                SELECT suppliers.name as supplier, email, countries.name as country
                FROM suppliers
                INNER JOIN countries
                ON suppliers.country_id = countries.country_id
                LIMIT 10
            ",
            &[],
        )?
        .iter()
        .map(|row| (row.get("supplier"), row.get("email"), row.get("country")))
        .collect();

    println!(
        "
First 10 suppliers with name, email and country
{:50} {:30} {}
",
        "Supplier", "Email", "Country"
    );

    for (supplier, email, country) in suppliers {
        println!("{supplier:50} {email:30} {country}")
    }

    let groceries: Vec<(String, String)> = client
        .query(
            "
            SELECT subcategory, products.name as product
            FROM 
            (   
                SELECT product_subcategories.name as subcategory, subcategory_id 
                FROM product_subcategories
                WHERE category_id = (
                    SELECT category_id from product_categories
                    WHERE name = 'Grocery'
                )
            ) as grocery_subcategories
            INNER JOIN products
            ON grocery_subcategories.subcategory_id = products.subcategory_id
            WHERE subcategory LIKE 'M%'
            ORDER by subcategory
        ",
            &[],
        )?
        .iter()
        .map(|row| (row.get("subcategory"), row.get("product")))
        .collect();

    println!(
        "
Groceries in subcategories that starts with M
{:50} {}
",
        "Subcategory", "Product"
    );

    for (subcategory, product) in groceries {
        println!("{subcategory:50} {product}")
    }

    let count: i64 = client.query(
        "
            SELECT COUNT(1)
            FROM users
        ",
        &[],
    )?[0]
        .get(0);

    println!();
    println!("count of users: {count}");

    let deleted = client.execute(
        "
        DELETE FROM users
        WHERE password = crypt('password', password) AND supplier_id IS NOT NULL
    ",
        &[],
    )?;

    println!("Deleted all supplier users with password 'password' (total {deleted})");

    let count: i64 = client.query(
        "
            SELECT COUNT(1)
            FROM users
        ",
        &[],
    )?[0]
        .get(0);

    println!("count of users: {count}");

    println!();
    println!("change product categories to lowercase");

    client.batch_execute(
        "
        UPDATE product_categories
        SET name = lower(name);
        UPDATE product_subcategories
        SET name = lower(name);
    ",
    )?;

    let caterories: Vec<(String, String)> = client
        .query(
            "
                SELECT product_categories.name as category, product_subcategories.name as subcategory
                FROM product_categories
                INNER JOIN product_subcategories 
                ON product_categories.category_id = product_subcategories.category_id
                WHERE product_categories.name NOT IN ('grocery', 'healthy eating')
            ",
            &[],
        )?
        .iter()
        .map(|row| (row.get("category"), row.get("subcategory")))
        .collect();

    println!(
        "
Subcategories per each category except 'grocery' and 'healthy eating'
Categories                     Subcategories
"
    );
    for (category, subcategory) in caterories {
        println!("{category:30} {subcategory}")
    }

    Ok(())
}

fn main() {
    let mut client = connect();

    drop_tables(&mut client);

    create_tables(&mut client).expect("can't create tables");

    create_indexes(&mut client).expect("can't create indexes");

    init_tables(&mut client).expect("can't init tables");

    demo_queries(&mut client).expect("can't show demo queries");
}
