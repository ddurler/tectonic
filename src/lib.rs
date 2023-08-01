//! `tectonic` permet de créer une grille de 'tectonic' et de la résoudre.
//!
//! `Tectonic` est un jeu de logique.
//!
//! Il faut compléter une grille avec les chiffres manquants dans chaque zone entourés de gras, sachant que :
//!
//! 1. Une zone de deux cases contient les chiffres 1 et 2, une zone de 3 cases les chiffres 1, 2 et 3, etc.
//! 2. Un chiffre placé dans une case ne peut se retrouver dans aucune des cases qui l'entoure (en diagonale y compris).
//!
//! La structure `Grid` permet de construire la grille.
//!
//! Lors de cette construction, une zone est repérée par une lettre ('a', 'b', etc.), une case est repérée
//! par une lettre (la zone qui contient cette case) et le chiffre qu'elle contient ou la zone seulement si
//! le chiffre de la case n'est pas encore connu.
//! //!
//! La structure `Solver` permet de résoudre cette grille
//!
//! ```rust
//! use std::str::FromStr;
//! use tectonic::{Grid, Solver};
//!
//! // Création de la grille
//! let grid = Grid::from_str(
//!     "
//! a1 b  b2
//! b4 b  b
//! c  c  c2
//! ",
//! )
//! .unwrap();
//!
//! println!("{grid}");
//!
//! // Résolution de la grille
//! let mut solver = Solver::new(&grid);
//! let _ = solver.solve();
//! println!("{solver}");
//! ```
mod grid;
mod line_column;
mod neighboring_line_columns;
mod solver;

pub use grid::{Grid, ParseGridError};
pub use solver::{Solver, SolvingError};
