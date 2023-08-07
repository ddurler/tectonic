use std::env;
use std::fs;
use std::str::FromStr;

use tectonic::{Grid, Solver};

pub fn main() {
    // Arguments de la line de commande
    let args: Vec<String> = env::args().collect();

    if args.len() == 2 {
        // Un nom de fichier passé en paramètre
        solve_grid_in_file(&args[1]);
    } else {
        // Aide utilisateur
        help();
    }
}

fn help() {
    println!("
Solver de grille 'Tectonic'.

`Tectonic` est un jeu de logique.
Ce jeu est également connu sous le nom de 'Suguru' ou 'Kemaru'.

Il faut compléter une grille avec les chiffres manquants dans chaque zone entourée de gras, sachant que :

1. Une zone de deux cases contient les chiffres 1 et 2, une zone de 3 cases les chiffres 1, 2 et 3, etc.
2. Un chiffre placé dans une case ne peut se retrouver dans aucune des cases qui l'entoure (en diagonale y compris).

La grille à résoudre doit être définie dans un fichier au format texte passé en paramètre.

Dans ce fichier, une zone est repérée par une lettre ('a', 'b', etc.) et chaque case est repérée
par une lettre (la zone qui contient cette case) et le chiffre qu'elle contient ou la zone seulement si
le chiffre de la case n'est pas encore connu.

Les lignes 'vides' ou qui commencent par un '#' (commentaires) sont ignorées.
    ");

    println!("Exemple d'utilisation :\n");
    example();
}

// Exemple d'utilisation
fn example() {
    let file_content = "
# Exemple
a1 b  b2
b4 b  b
c  c  c2
";

    println!("Le fichier de définition de la grille contient le texte suivant :");
    println!("{file_content}");

    println!("La résolution de cette grille est alors :\n");
    let grid = Grid::from_str(file_content).unwrap();
    let mut solver = Solver::new(&grid);
    let _ = solver.solve(|action| println!("{action}"));
    println!("{solver}");
}

// Résolution d'une grille définie dans un fichier
fn solve_grid_in_file(path: &str) {
    println!("Lecture de '{path}'...");
    match fs::read_to_string(path) {
        Err(e) => println!("Erreur de lecture du fichier '{path}': {e}\n"),
        Ok(file_content) => match Grid::from_str(&file_content) {
            Err(e) => println!("Erreur dans le fichier '{path}': {e}\n"),
            Ok(grid) => {
                let mut solver = Solver::new(&grid);
                let res_solver = solver.solve(|action| println!("{action}"));
                match res_solver {
                    Err(e) => println!("Erreur résolution avec le fichier '{path}': {e}\n"),
                    Ok(done) => {
                        if done {
                            println!("Résolu ({})", solver.difficulty_level);
                        } else {
                            println!("(Non résolu :(");
                        }
                        println!("{solver}");
                    }
                }
            }
        },
    }
}
