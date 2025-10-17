use chrono::{DateTime, Local};
use rpassword::read_password;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{self, Write};

const DATA_FILE: &str = "store_data.json";
const DEFAULT_ADMIN_USER: &str = "admin";
const DEFAULT_ADMIN_PASS: &str = "password";

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Product {
    id: u32,
    name: String,
    description: String,
    price: f64,
    quantity: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Sale {
    id: u32,
    product_id: u32,
    quantity: i32,
    sale_price: f64,
    time: DateTime<Local>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Purchase {
    id: u32,
    product_id: u32,
    quantity: i32,
    purchase_price: f64,
    time: DateTime<Local>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Manager {
    username: String,
    password_hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Store {
    products: Vec<Product>,
    sales: Vec<Sale>,
    purchases: Vec<Purchase>,
    managers: Vec<Manager>,
    next_product_id: u32,
    next_sale_id: u32,
    next_purchase_id: u32,
}

#[derive(Debug)]
enum StoreError {
    NotFound(String),
    InsufficientStock(String),
    InvalidInput(String),
    IoError(String),
}

impl Store {
    fn new() -> Self {
        let mut s = Store {
            products: Vec::new(),
            sales: Vec::new(),
            purchases: Vec::new(),
            managers: Vec::new(),
            next_product_id: 1,
            next_sale_id: 1,
            next_purchase_id: 1,
        };
        if s.managers.is_empty() {
            let default_hash = hash_password(DEFAULT_ADMIN_PASS);
            s.managers.push(Manager {
                username: DEFAULT_ADMIN_USER.to_string(),
                password_hash: default_hash,
            });
        }
        s
    }

    fn add_product(
        &mut self,
        name: String,
        description: String,
        price: f64,
        quantity: i32,
    ) -> Product {
        let product = Product {
            id: self.next_product_id,
            name,
            description,
            price,
            quantity,
        };
        self.next_product_id += 1;
        self.products.push(product.clone());
        product
    }

    fn edit_product(
        &mut self,
        id: u32,
        name: Option<String>,
        description: Option<String>,
        price: Option<f64>,
        quantity: Option<i32>,
    ) -> Result<Product, StoreError> {
        match self.products.iter_mut().find(|p| p.id == id) {
            Some(p) => {
                if let Some(n) = name {
                    p.name = n;
                }
                if let Some(d) = description {
                    p.description = d;
                }
                if let Some(pr) = price {
                    p.price = pr;
                }
                if let Some(q) = quantity {
                    p.quantity = q;
                }
                Ok(p.clone())
            }
            None => Err(StoreError::NotFound(format!("Product {} not found", id))),
        }
    }

    fn delete_product(&mut self, id: u32) -> Result<(), StoreError> {
        let idx = self.products.iter().position(|p| p.id == id);
        if let Some(i) = idx {
            self.products.remove(i);
            Ok(())
        } else {
            Err(StoreError::NotFound(format!("Product {} not found", id)))
        }
    }

    fn record_purchase(
        &mut self,
        product_id: u32,
        quantity: i32,
        purchase_price: f64,
    ) -> Result<Purchase, StoreError> {
        if quantity <= 0 {
            return Err(StoreError::InvalidInput("Quantity must be positive".into()));
        }
        let product = match self.products.iter_mut().find(|p| p.id == product_id) {
            Some(p) => p,
            None => return Err(StoreError::NotFound(format!("Product {} not found", product_id))),
        };
        product.quantity += quantity;
        let pur = Purchase {
            id: self.next_purchase_id,
            product_id,
            quantity,
            purchase_price,
            time: Local::now(),
        };
        self.next_purchase_id += 1;
        self.purchases.push(pur.clone());
        Ok(pur)
    }

    fn record_sale(
        &mut self,
        product_id: u32,
        quantity: i32,
        sale_price: f64,
    ) -> Result<Sale, StoreError> {
        if quantity <= 0 {
            return Err(StoreError::InvalidInput("Quantity must be positive".into()));
        }
        let product = match self.products.iter_mut().find(|p| p.id == product_id) {
            Some(p) => p,
            None => return Err(StoreError::NotFound(format!("Product {} not found", product_id))),
        };
        if product.quantity < quantity {
            return Err(StoreError::InsufficientStock(format!(
                "{} has only {} in stock",
                product.name, product.quantity
            )));
        }
        product.quantity -= quantity;
        let sale = Sale {
            id: self.next_sale_id,
            product_id,
            quantity,
            sale_price,
            time: Local::now(),
        };
        self.next_sale_id += 1;
        self.sales.push(sale.clone());
        Ok(sale)
    }

    fn total_sales(&self) -> f64 {
        self.sales.iter().map(|s| s.sale_price * s.quantity as f64).sum()
    }

    fn total_purchases_cost(&self) -> f64 {
        self.purchases
            .iter()
            .map(|p| p.purchase_price * p.quantity as f64)
            .sum()
    }

    fn profit(&self) -> f64 {
        self.total_sales() - self.total_purchases_cost()
    }

    fn find_product(&self, id: u32) -> Option<&Product> {
        self.products.iter().find(|p| p.id == id)
    }

    fn save_to_file(&self) -> Result<(), StoreError> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| StoreError::IoError(format!("Serialize error: {}", e)))?;
        fs::write(DATA_FILE, json)
            .map_err(|e| StoreError::IoError(format!("Write error: {}", e)))?;
        Ok(())
    }

    fn load_from_file() -> Result<Self, StoreError> {
        match fs::read_to_string(DATA_FILE) {
            Ok(s) => serde_json::from_str(&s)
                .map_err(|e| StoreError::IoError(format!("Deserialize error: {}", e))),
            Err(_) => {
                let mut st = Store::new();
                if st.managers.is_empty() {
                    st.managers.push(Manager {
                        username: DEFAULT_ADMIN_USER.to_string(),
                        password_hash: hash_password(DEFAULT_ADMIN_PASS),
                    });
                }
                Ok(st)
            }
        }
    }

    fn add_manager(&mut self, username: &str, password: &str) {
        let hash = hash_password(password);
        self.managers.push(Manager {
            username: username.to_string(),
            password_hash: hash,
        });
    }

    fn authenticate(&self, username: &str, password: &str) -> bool {
        let hash = hash_password(password);
        self.managers
            .iter()
            .any(|m| m.username == username && m.password_hash == hash)
    }
}

fn hash_password(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    let res = hasher.finalize();
    format!("{:x}", res)
}

fn prompt(msg: &str) -> String {
    print!("{}", msg);
    let _ = io::stdout().flush();
    let mut buf = String::new();
    io::stdin()
        .read_line(&mut buf)
        .expect("Failed to read input");
    buf.trim().to_string()
}

fn pause() {
    let _ = prompt("\nPress Enter to continue...");
}

fn main_menu() {
    println!("Welcome to Rusty Store Inventory Management System");
    println!("Loading data...");
}

fn ui_loop(mut store: Store) {
    loop {
        println!("\n--- Main Menu ---");
        println!("1. Inventory Management");
        println!("2. Sales Management");
        println!("3. Purchase Management");
        println!("4. Reports");
        println!("5. Save & Exit");
        let choice = prompt("Select option: ");
        match choice.as_str() {
            "1" => inventory_menu(&mut store),
            "2" => sales_menu(&mut store),
            "3" => purchases_menu(&mut store),
            "4" => reports_menu(&mut store),
            "5" => {
                match store.save_to_file() {
                    Ok(_) => println!("Data saved to {}", DATA_FILE),
                    Err(e) => eprintln!("Error saving: {:?}", e),
                }
                println!("Goodbye!");
                break;
            }
            _ => println!("Invalid selection."),
        }
    }
}

fn inventory_menu(store: &mut Store) {
    loop {
        println!("\n--- Inventory Menu ---");
        println!("1. List products");
        println!("2. Add product");
        println!("3. Edit product");
        println!("4. Delete product");
        println!("5. Back");
        let choice = prompt("Select option: ");
        match choice.as_str() {
            "1" => {
                println!("\nInventory:");
                for p in &store.products {
                    println!(
                        "[{}] {} - {} | ${:.2} | qty: {}",
                        p.id, p.name, p.description, p.price, p.quantity
                    );
                }
                pause();
            }
            "2" => {
                let name = prompt("Name: ");
                let description = prompt("Description: ");
                let price_s = prompt("Price: ");
                let qty_s = prompt("Quantity: ");
                match (price_s.parse::<f64>(), qty_s.parse::<i32>()) {
                    (Ok(price), Ok(qty)) => {
                        let pr = store.add_product(name, description, price, qty);
                        println!("Product added: {:?}", pr);
                    }
                    _ => println!("Invalid price or quantity."),
                }
                pause();
            }
            "3" => {
                let id_s = prompt("Product id to edit: ");
                if let Ok(id) = id_s.parse::<u32>() {
                    let name = prompt("New name (or empty to skip): ");
                    let desc = prompt("New description (or empty to skip): ");
                    let price_s = prompt("New price (or empty to skip): ");
                    let qty_s = prompt("New quantity (or empty to skip): ");
                    let name_opt = if name.is_empty() { None } else { Some(name) };
                    let desc_opt = if desc.is_empty() { None } else { Some(desc) };
                    let price_opt = if price_s.is_empty() {
                        None
                    } else {
                        match price_s.parse::<f64>() {
                            Ok(v) => Some(v),
                            Err(_) => {
                                println!("Invalid price");
                                None
                            }
                        }
                    };
                    let qty_opt = if qty_s.is_empty() {
                        None
                    } else {
                        match qty_s.parse::<i32>() {
                            Ok(v) => Some(v),
                            Err(_) => {
                                println!("Invalid quantity");
                                None
                            }
                        }
                    };
                    match store.edit_product(id, name_opt, desc_opt, price_opt, qty_opt) {
                        Ok(p) => println!("Updated: {:?}", p),
                        Err(e) => println!("Error: {:?}", e),
                    }
                } else {
                    println!("Invalid id");
                }
                pause();
            }
            "4" => {
                let id_s = prompt("Product id to delete: ");
                if let Ok(id) = id_s.parse::<u32>() {
                    match store.delete_product(id) {
                        Ok(_) => println!("Deleted product {}", id),
                        Err(e) => println!("Error: {:?}", e),
                    }
                } else {
                    println!("Invalid id");
                }
                pause();
            }
            "5" => break,
            _ => println!("Invalid selection"),
        }
    }
}

fn sales_menu(store: &mut Store) {
    loop {
        println!("\n--- Sales Menu ---");
        println!("1. Record sale");
        println!("2. List sales");
        println!("3. Back");
        let choice = prompt("Select option: ");
        match choice.as_str() {
            "1" => {
                let pid_s = prompt("Product id: ");
                let qty_s = prompt("Quantity: ");
                let price_s = prompt("Sale price per unit: ");
                match (pid_s.parse::<u32>(), qty_s.parse::<i32>(), price_s.parse::<f64>()) {
                    (Ok(pid), Ok(qty), Ok(price)) => match store.record_sale(pid, qty, price) {
                        Ok(sale) => {
                            println!("Recorded sale: {:?}", sale);
                            let profit = sale.sale_price * sale.quantity as f64;
                            println!("Total sale amount: ${:.2}", profit);
                        }
                        Err(e) => println!("Error: {:?}", e),
                    },
                    _ => println!("Invalid input"),
                }
                pause();
            }
            "2" => {
                println!("\nSales history:");
                for s in &store.sales {
                    if let Some(prod) = store.find_product(s.product_id) {
                        println!(
                            "[{}] {} x{} @ ${:.2} each = ${:.2} at {}",
                            s.id,
                            prod.name,
                            s.quantity,
                            s.sale_price,
                            s.sale_price * s.quantity as f64,
                            s.time
                        );
                    }
                }
                println!("Total sales: ${:.2}", store.total_sales());
                pause();
            }
            "3" => break,
            _ => println!("Invalid selection"),
        }
    }
}

fn purchases_menu(store: &mut Store) {
    loop {
        println!("\n--- Purchases Menu ---");
        println!("1. Record purchase");
        println!("2. List purchases");
        println!("3. Back");
        let choice = prompt("Select option: ");
        match choice.as_str() {
            "1" => {
                let pid_s = prompt("Product id: ");
                let qty_s = prompt("Quantity: ");
                let price_s = prompt("Purchase price per unit: ");
                match (pid_s.parse::<u32>(), qty_s.parse::<i32>(), price_s.parse::<f64>()) {
                    (Ok(pid), Ok(qty), Ok(price)) => match store.record_purchase(pid, qty, price) {
                        Ok(pur) => {
                            println!("Recorded purchase: {:?}", pur);
                            println!("Total cost: ${:.2}", pur.purchase_price * pur.quantity as f64);
                        }
                        Err(e) => println!("Error: {:?}", e),
                    },
                    _ => println!("Invalid input"),
                }
                pause();
            }
            "2" => {
                println!("\nPurchase history:");
                for p in &store.purchases {
                    if let Some(prod) = store.find_product(p.product_id) {
                        println!(
                            "[{}] {} x{} @ ${:.2} each = ${:.2} at {}",
                            p.id,
                            prod.name,
                            p.quantity,
                            p.purchase_price,
                            p.purchase_price * p.quantity as f64,
                            p.time
                        );
                    }
                }
                println!("Total purchases cost: ${:.2}", store.total_purchases_cost());
                pause();
            }
            "3" => break,
            _ => println!("Invalid selection"),
        }
    }
}

fn reports_menu(store: &Store) {
    loop {
        println!("\n--- Reports Menu ---");
        println!("1. Inventory report");
        println!("2. Sales & Profit summary");
        println!("3. Purchase history");
        println!("4. Full report (all)");
        println!("5. Back");
        let choice = prompt("Select option: ");
        match choice.as_str() {
            "1" => {
                println!("\nInventory Report:");
                println!("{:<5} {:<20} {:<8} {:<6} {}", "ID", "Name", "Price", "Qty", "Description");
                for p in &store.products {
                    println!(
                        "{:<5} {:<20} ${:<7.2} {:<6} {}",
                        p.id, p.name, p.price, p.quantity, p.description
                    );
                }
                pause();
            }
            "2" => {
                println!("\nSales Summary:");
                println!("Total Sales: ${:.2}", store.total_sales());
                println!("Total Purchases Cost: ${:.2}", store.total_purchases_cost());
                println!("Estimated Profit: ${:.2}", store.profit());
                pause();
            }
            "3" => {
                println!("\nPurchases:");
                for p in &store.purchases {
                    println!(
                        "[{}] Product {} qty {} @ ${:.2} on {}",
                        p.id, p.product_id, p.quantity, p.purchase_price, p.time
                    );
                }
                pause();
            }
            "4" => {
                println!("\n--- FULL REPORT ---");
                println!("Inventory:");
                for p in &store.products {
                    println!(
                        "[{}] {} — ${:.2} — qty {}",
                        p.id, p.name, p.price, p.quantity
                    );
                }
                println!("\nSales:");
                for s in &store.sales {
                    println!(
                        "[{}] product {} qty {} @ ${:.2} each — total ${:.2} — {}",
                        s.id,
                        s.product_id,
                        s.quantity,
                        s.sale_price,
                        s.sale_price * s.quantity as f64,
                        s.time
                    );
                }
                println!("\nPurchases:");
                for p in &store.purchases {
                    println!(
                        "[{}] product {} qty {} @ ${:.2} each — total ${:.2} — {}",
                        p.id,
                        p.product_id,
                        p.quantity,
                        p.purchase_price,
                        p.purchase_price * p.quantity as f64,
                        p.time
                    );
                }
                println!("\nSummary:");
                println!("Total Sales: ${:.2}", store.total_sales());
                println!("Total Purchases Cost: ${:.2}", store.total_purchases_cost());
                println!("Profit: ${:.2}", store.profit());
                pause();
            }
            "5" => break,
            _ => println!("Invalid selection"),
        }
    }
}

fn login_sequence() -> bool {
    println!("Please login as manager to continue.");
    let username = prompt("Username: ");
    print!("Password: ");
    let _ = io::stdout().flush();
    let password = read_password().unwrap_or_else(|_| prompt("Password (fallback): "));
    match Store::load_from_file() {
        Ok(store) => {
            if store.authenticate(&username, &password) {
                println!("Login success. Welcome, {}!", username);
                true
            } else {
                println!("Login failed.");
                false
            }
        }
        Err(e) => {
            println!("Failed to load data (proceeding): {:?}", e);
            false
        }
    }
}

fn main() {
    main_menu();
    let store = match Store::load_from_file() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to load data: {:?}. Starting with empty store.", e);
            let mut s = Store::new();
            s
        }
    };
    if !login_sequence() {
        println!("Exiting due to authentication failure.");
        return;
    }
    let store = Store::load_from_file().unwrap_or_else(|_| Store::new());
    ui_loop(store);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_edit_delete_product() {
        let mut store = Store::new();
        let p = store.add_product("P".into(), "D".into(), 9.99, 10);
        assert_eq!(p.id, 1);
        let edited = store
            .edit_product(p.id, Some("P2".into()), None, Some(10.0), Some(5))
            .unwrap();
        assert_eq!(edited.name, "P2");
        assert_eq!(edited.price, 10.0);
        assert_eq!(edited.quantity, 5);
        assert!(store.delete_product(p.id).is_ok());
        assert!(store.delete_product(999).is_err());
    }

    #[test]
    fn purchase_and_sales() {
        let mut store = Store::new();
        let p = store.add_product("A".into(), "desc".into(), 5.0, 2);
        let pur = store.record_purchase(p.id, 10, 4.0).unwrap();
        assert_eq!(pur.quantity, 10);
        assert!((store.total_purchases_cost() - 40.0).abs() < 1e-6);
        let sale = store.record_sale(p.id, 5, 7.0).unwrap();
        assert_eq!(sale.quantity, 5);
        let prod = store.find_product(p.id).unwrap();
        assert_eq!(prod.quantity, 7); 
    }

    #[test]
    fn auth_and_manager() {
        let mut store = Store::new();
        store.add_manager("test", "1234");
        assert!(store.authenticate("test", "1234"));
        assert!(!store.authenticate("test", "wrong"));
    }
}
