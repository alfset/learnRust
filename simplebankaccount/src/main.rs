use std::io;

trait Account {
    fn deposit(&mut self, amount: f64) -> Result<(), String>;
    fn withdraw(&mut self, amount: f64) -> Result<(), String>;
    fn balance(&self) -> f64;
}

struct BankAccount {
    account_number: u32,
    holder_name: String,
    balance: f64,
}

impl Account for BankAccount {
    fn deposit(&mut self, amount: f64) -> Result<(), String> {
        if amount <= 0.0 {
            return Err("Deposit amount must be greater than zero.".to_string());
        }

        self.balance += amount;
        println!(
            "Deposited ${:.2} into account {} ({}) — New balance: ${:.2}",
            amount, self.account_number, self.holder_name, self.balance
        );
        Ok(())
    }

    fn withdraw(&mut self, amount: f64) -> Result<(), String> {
        if amount <= 0.0 {
            return Err("Withdrawal amount must be greater than zero.".to_string());
        }
        if amount > self.balance {
            return Err(format!(
                "Insufficient funds in account {} ({}). Current balance: ${:.2}",
                self.account_number, self.holder_name, self.balance
            ));
        }

        self.balance -= amount;
        println!(
            "Withdrew ${:.2} from account {} ({}) — New balance: ${:.2}",
            amount, self.account_number, self.holder_name, self.balance
        );
        Ok(())
    }

    fn balance(&self) -> f64 {
        self.balance
    }
}

fn main() {
    let mut accounts = vec![
        BankAccount {
            account_number: 1001,
            holder_name: String::from("Alice"),
            balance: 500.0,
        },
        BankAccount {
            account_number: 1002,
            holder_name: String::from("Bob"),
            balance: 1000.0,
        },
    ];

    loop {
        println!("\n===== Banking System =====");
        println!("1. Deposit");
        println!("2. Withdraw");
        println!("3. Check Balance");
        println!("4. List Accounts");
        println!("5. Exit");
        print!("Choose an option (1-5): ");

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).expect("Failed to read input");
        let choice = choice.trim();

        match choice {
            "1" => {
                if let Some(account) = select_account(&mut accounts) {
                    println!("Enter deposit amount:");
                    let mut amount_input = String::new();
                    io::stdin().read_line(&mut amount_input).expect("Failed to read input");
                    if let Ok(amount) = amount_input.trim().parse::<f64>() {
                        match account.deposit(amount) {
                            Ok(_) => println!("Deposit successful!"),
                            Err(e) => println!("Error: {}", e),
                        }
                    } else {
                        println!("Invalid amount entered.");
                    }
                }
            }
            "2" => {
                if let Some(account) = select_account(&mut accounts) {
                    println!("Enter withdrawal amount:");
                    let mut amount_input = String::new();
                    io::stdin().read_line(&mut amount_input).expect("Failed to read input");
                    if let Ok(amount) = amount_input.trim().parse::<f64>() {
                        match account.withdraw(amount) {
                            Ok(_) => println!("Withdrawal successful!"),
                            Err(e) => println!("Error: {}", e),
                        }
                    } else {
                        println!("Invalid amount entered.");
                    }
                }
            }
            "3" => {
                if let Some(account) = select_account(&mut accounts) {
                    println!(
                        "Account {} ({}) balance: ${:.2}",
                        account.account_number,
                        account.holder_name,
                        account.balance()
                    );
                }
            }
            "4" => {
                println!("\n=== Account List ===");
                for acc in &accounts {
                    println!(
                        "Account {} — {} — Balance: ${:.2}",
                        acc.account_number, acc.holder_name, acc.balance
                    );
                }
            }
            "5" => {
                println!("Goodbye!");
                break;
            }
            _ => println!("Invalid option, please try again."),
        }
    }
}

fn select_account<'a>(accounts: &'a mut Vec<BankAccount>) -> Option<&'a mut BankAccount> {
    println!("Enter account number:");
    let mut acc_input = String::new();
    io::stdin().read_line(&mut acc_input).expect("Failed to read input");

    if let Ok(acc_number) = acc_input.trim().parse::<u32>() {
        for acc in accounts.iter_mut() {
            if acc.account_number == acc_number {
                println!(
                    "Selected account {} ({})",
                    acc.account_number, acc.holder_name
                );
                return Some(acc);
            }
        }
        println!("Account number {} not found.", acc_number);
        None
    } else {
        println!("Invalid account number entered.");
        None
    }
}
