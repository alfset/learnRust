use std::io;

#[derive(Debug, Clone)]
struct Player {
    name: String,
    position: String,
}

struct FilterCondition<F>
where
    F: Fn(&Player) -> bool,
{
    condition: F,
}
trait MatchCondition<T> {
    fn is_match(&self, item: &T) -> bool;
}
impl<F> MatchCondition<Player> for FilterCondition<F>
where
    F: Fn(&Player) -> bool,
{
    fn is_match(&self, item: &Player) -> bool {
        (self.condition)(item)
    }
}

fn custom_filter<F>(collection: &[Player], filter: &FilterCondition<F>) -> Vec<Player>
where
    F: Fn(&Player) -> bool,
{
    collection
        .iter()
        .filter(|item| filter.is_match(item))
        .cloned()
        .collect()
}

fn main() {
    let players = vec![
        Player { name: "Neuer".to_string(), position: "GK".to_string() },
        Player { name: "Ramos".to_string(), position: "CB".to_string() },
        Player { name: "Modric".to_string(), position: "CMF".to_string() },
        Player { name: "De Bruyne".to_string(), position: "AMF".to_string() },
        Player { name: "Haaland".to_string(), position: "CF".to_string() },
        Player { name: "Kane".to_string(), position: "CF".to_string() },
    ];

    loop {
        println!("\n=== Player Filter Menu ===");
        println!("1. Show all players");
        println!("2. Filter by position (GK, CB, CMF, AMF, CF)");
        println!("3. Exit");
        print!("Enter choice: ");

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).expect("Failed to read input");

        match choice.trim() {
            "1" => {
                println!("\nAll Players:");
                for player in &players {
                    println!("{} - {}", player.name, player.position);
                }
            }
            "2" => {
                print!("Enter position to filter (e.g. CF): ");
                let mut pos = String::new();
                io::stdin().read_line(&mut pos).expect("Failed to read input");
                let pos = pos.trim().to_uppercase();
                let pos_clone = pos.clone(); 
                let filter = FilterCondition {
                    condition: move |p: &Player| p.position.eq_ignore_ascii_case(&pos),
                };
                let result = custom_filter(&players, &filter);
                if result.is_empty() {
                    println!("\nNo players found for position: {}", pos_clone);
                } else {
                    println!("\nPlayers in position {}:", pos_clone);
                    for player in result {
                        println!("{} - {}", player.name, player.position);
                    }
                }
            }
            "3" => {
                println!("Exiting...");
                break;
            }
            _ => println!("Invalid choice! Try again."),
        }
    }
}
