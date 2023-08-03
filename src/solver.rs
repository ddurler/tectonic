use std::collections::HashMap;
use std::fmt;

use crate::grid::{CellContent, Grid};
use crate::line_column::LineColumn;
use crate::neighboring_line_columns::NeighboringLineColumns;
use crate::simple_09_set::Simple09Set;

/// Action possible effectuée à chaque étape de résolution
#[derive(Debug, PartialEq, Eq)]
pub enum SolvingAction {
    /// La grille est résolue
    Solved,

    /// Initialisation des chiffres possibles pour toutes les cases
    InitPossibleNumbers,

    /// Case qu'une seule possibilité de chiffre
    SinglePossibleNumber(LineColumn, u8),

    /// Suppression des chiffres d'une case qui sont déjà dans la zone de cette case
    NumbersInZone(LineColumn, char, Vec<u8>),

    /// Suppression des chiffres d'une case qui sont déjà dans une de ses cases voisines
    NumbersNeighboring(LineColumn, Vec<u8>),

    /// Aucune action de résolution trouvée
    NoAction,
}

impl fmt::Display for SolvingAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Solved => {
                write!(f, "Grille résolue")
            }
            Self::InitPossibleNumbers => {
                write!(f, "Initialisation des chiffres possibles dans le cases...")
            }
            Self::SinglePossibleNumber(line_column, n) => {
                write!(f, "Seule possibilité pour la case {line_column}: '{n}")
            }
            Self::NumbersInZone(line_column, c_zone, vec_n) => {
                write!(
                    f,
                    "{vec_n:?} déjà dans la zone '{c_zone}' de la case {line_column}"
                )
            }
            Self::NumbersNeighboring(line_column, vec_n) => {
                write!(
                    f,
                    "{vec_n:?} déjà dans les cases voisines de la case {line_column}"
                )
            }
            SolvingAction::NoAction => {
                write!(f, "Aucune action de résolution trouvée")
            }
        }
    }
}

/// Cas d'erreurs possibles pendant la résolution de la grille tectonic
#[derive(Debug)]
pub enum SolvingError {
    /// Deux cases voisines avec le même chiffre
    NeighboringWithSameNumber(LineColumn, LineColumn, u8),

    /// Deux cases d'une même zone avec le même chiffre
    ZoneWithSameNumber(char, u8),

    /// Aucun chiffre possible pour une case
    NoPossibleNumber(LineColumn),

    /// Erreur d'implémentation qui ne devrait pas arriver :)
    BadImplementation,
}

impl fmt::Display for SolvingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SolvingError::NeighboringWithSameNumber(line_column_1, line_column_2, n) => {
                write!(f, "Le chiffre {n} apparaît dans les cases voisines {line_column_1:?} et {line_column_2:?}")
            }
            Self::ZoneWithSameNumber(c_zone, n) => {
                write!(
                    f,
                    "Le chiffre {n} apparaît plusieurs fois dans la zone '{c_zone}'"
                )
            }
            Self::NoPossibleNumber(line_column) => {
                write!(f, "Aucun chiffre possible dans la case {line_column:?}")
            }
            SolvingError::BadImplementation => write!(f, "Erreur inattendue (voir source code...)"),
        }
    }
}

impl std::error::Error for SolvingError {}

/// Structure pour la résolution d'une grille tectonic
#[derive(Debug, Default)]
pub struct Solver {
    grid: Grid,

    /// True lorsque toutes les cases avec un contenu `Undefined` ont été traitées
    init_cell_contents: bool,
}

impl fmt::Display for Solver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.grid)
    }
}

impl Solver {
    /// Constructeur de l'algorithme de résolution d'après une grille
    /// Il faut utiliser en suite l'une des primitives de résolution :
    /// * `solve` : Pour rechercher une solution
    /// * `solve_step` : Pour les différentes étapes de résolution
    #[must_use]
    pub fn new(grid: &Grid) -> Self {
        Solver {
            grid: grid.clone(),
            init_cell_contents: false,
        }
    }

    /// Retourne true si la grille est résolue
    #[must_use]
    pub fn is_solved(&self) -> bool {
        for cell in self.grid.hashmap_cells.values() {
            if let CellContent::Number(_) = cell.content {
                continue;
            }
            return false;
        }
        true
    }

    /// Tente de résoudre a grille en itérant continûment sur toutes les étapes de résolution
    /// Retourne true si la grille est résolue
    /// # Errors
    /// Une erreur est retournée si la grille n'est pas (ou plus) cohérente
    pub fn solve(&mut self) -> Result<bool, SolvingError> {
        #[allow(while_true)]
        while true {
            let action_solve_step = self.solve_step()?;
            println!("{action_solve_step}");
            match action_solve_step {
                SolvingAction::Solved => return Ok(true),
                SolvingAction::NoAction => return Ok(false),
                _ => continue,
            }
        }
        Err(SolvingError::BadImplementation)
    }

    /// Applique une étape de résolution
    /// Retourne une action effectuée pour rechercher la solution
    /// Si `SolvingAction::Solved` est retourné, c'est que la grille est résolue
    /// Si `SolvingAction::NoAction` est retourné, c'est que l'algorithme a épuisé les
    /// étapes de sa recherche
    /// # Errors
    /// Une erreur est retournée si la grille n'est pas (ou plus) cohérente
    pub fn solve_step(&mut self) -> Result<SolvingAction, SolvingError> {
        // Vérifie la cohérence de la grille
        self.check()?;

        if !self.init_cell_contents {
            self.solve_step_possible_numbers();
            self.init_cell_contents = true;
            return Ok(SolvingAction::InitPossibleNumbers);
        }

        // Fonctions-actions pour la résolution

        let action = self.solve_single_possible_number();
        if let SolvingAction::NoAction = action {
        } else {
            return Ok(action);
        }

        let action = self.solve_numbers_in_zone();
        if let SolvingAction::NoAction = action {
        } else {
            return Ok(action);
        }

        let action = self.solve_numbers_neighboring();
        if let SolvingAction::NoAction = action {
        } else {
            return Ok(action);
        }

        Ok(SolvingAction::NoAction)
    }

    /// Etape initiale de résolution pour modifier toutes les cases avec un
    /// contenu `Undefined` en un contenu `PossibleNumbers` selon les chiffres
    /// déjà en place dans la zone
    fn solve_step_possible_numbers(&mut self) -> SolvingAction {
        // Prépare la liste des des chiffres possibles par zone
        let mut zone_hash_map: HashMap<char, Simple09Set> = HashMap::new();
        for (c_zone, zone) in &self.grid.hashmap_zones {
            let nb_cases = zone.set_line_column.len();
            let mut simple_09_set = Simple09Set::default();
            // On ne considère que le nombre de cases de la zone pour la liste
            // des chiffres possibles dans les cases de cette zone
            for n in 1..=nb_cases {
                #[allow(clippy::cast_possible_truncation)]
                simple_09_set.insert(n as u8);
            }
            zone_hash_map.insert(*c_zone, simple_09_set);
        }
        // Recherche de toutes les cases avec un contenu 'Undefined'
        for cell in self.grid.hashmap_cells.values_mut() {
            if let CellContent::Undefined = cell.content {
                // Case à traiter, encore à Undefined...
                let simple_09_set = zone_hash_map.get(&cell.c_zone).unwrap();
                cell.content = CellContent::PossibleNumbers(*simple_09_set);
            }
        }

        SolvingAction::NoAction
    }

    /// Etape pour identifier les cases qui n'ont qu'une seule possibilité pour le chiffre
    fn solve_single_possible_number(&mut self) -> SolvingAction {
        // Recherche de toutes les cases avec un contenu 'Undefined'
        for cell in self.grid.hashmap_cells.values_mut() {
            if let CellContent::PossibleNumbers(simple_09_set) = cell.content.clone() {
                if simple_09_set.len() == 1 {
                    let vec_n = simple_09_set.as_vec_u8();
                    let n = vec_n[0];
                    cell.content = CellContent::Number(n);
                    return SolvingAction::SinglePossibleNumber(cell.line_column, n);
                }
            }
        }

        SolvingAction::NoAction
    }

    /// Etape pour éliminer les chiffres déjà présents dans la zone d'une case
    fn solve_numbers_in_zone(&mut self) -> SolvingAction {
        // Prépare la liste des chiffres déjà placés par zone
        let mut zone_hash_map: HashMap<char, Simple09Set> = HashMap::new();
        for (c_zone, zone) in &self.grid.hashmap_zones {
            let mut simple_09_set = Simple09Set::default();
            for line_column in &zone.set_line_column {
                let cell = self.grid.get_cell(*line_column).unwrap();
                if let CellContent::Number(n) = cell.content {
                    simple_09_set.insert(n);
                }
            }
            zone_hash_map.insert(*c_zone, simple_09_set);
        }

        // Recherche de toutes les cases avec un contenu 'PossibleNumbers'
        for cell in self.grid.hashmap_cells.values_mut() {
            if let CellContent::PossibleNumbers(cell_simple_09_set) = cell.content.clone() {
                let c_zone = cell.c_zone;
                let mut simple_09_set = *zone_hash_map.get(&c_zone).unwrap();
                simple_09_set.intersection(cell_simple_09_set);
                if !simple_09_set.is_empty() {
                    // les valeurs dans simple_09_set sont déjà affectées à d'autres cases
                    // de la zone. Elles ne sont pas possible pour cette case
                    let vec_n = simple_09_set.as_vec_u8();
                    let mut new_cell_simple_09_set = cell_simple_09_set;
                    for n in &vec_n {
                        new_cell_simple_09_set.remove(*n);
                    }
                    cell.content = CellContent::PossibleNumbers(new_cell_simple_09_set);
                    return SolvingAction::NumbersInZone(cell.line_column, c_zone, vec_n);
                }
            }
        }

        SolvingAction::NoAction
    }

    /// Etape pour éliminer les chiffres déjà présents dans les cases voisines
    fn solve_numbers_neighboring(&mut self) -> SolvingAction {
        // Liste des cases avec un contenu 'PossibleNumbers'
        let mut vec_line_columns_possible_numbers: Vec<(LineColumn, Simple09Set)> = Vec::new();
        for cell in self.grid.hashmap_cells.values() {
            if let CellContent::PossibleNumbers(simple_09_set) = cell.content {
                vec_line_columns_possible_numbers.push((cell.line_column, simple_09_set));
            }
        }

        // Parcourt des cases avec un contenu 'PossibleNumbers'
        for (cell_line_column, cell_simple_09_set) in vec_line_columns_possible_numbers {
            // simple_09_set des cases voisines
            let mut neighboring_simple_09_set = Simple09Set::default();
            // Parcourt des cases voisines
            let neighboring_line_columns = NeighboringLineColumns::new(
                cell_line_column,
                self.grid.min_line_column,
                self.grid.max_line_column,
            );
            for neighboring_line_column in neighboring_line_columns {
                let option_cell = self.grid.get_cell(neighboring_line_column);
                if let Some(neighboring_cell) = option_cell {
                    if let CellContent::Number(neighboring_n) = neighboring_cell.content {
                        // Simple_09_set des chiffres dans les cases voisines
                        neighboring_simple_09_set.insert(neighboring_n);
                    }
                }
            }

            let mut intersection_simple_09set = cell_simple_09_set;
            intersection_simple_09set.intersection(neighboring_simple_09_set);
            if !intersection_simple_09set.is_empty() {
                // les valeurs dans intersection_simple_09set sont déjà affectées à des cases voisines
                // Elles ne sont pas possible pour cette case en line_column
                let cell = self.grid.get_mut_cell(cell_line_column).unwrap();
                let vec_n = intersection_simple_09set.as_vec_u8();
                let mut new_cell_simple_09_set = cell_simple_09_set;
                for n in &vec_n {
                    new_cell_simple_09_set.remove(*n);
                }
                cell.content = CellContent::PossibleNumbers(new_cell_simple_09_set);
                return SolvingAction::NumbersNeighboring(cell.line_column, vec_n);
            }
        }

        SolvingAction::NoAction
    }

    /// Vérifie la consistance de la grille
    fn check(&self) -> Result<(), SolvingError> {
        self.check_neighboring_cells()?;
        self.check_zone_numbers()?;
        self.check_cell_with_no_possible_values()?;
        Ok(())
    }

    /// Vérifie que pour toutes les cases avec un chiffre défini, il n'y a pas une case voisine
    /// définie avec le même chiffre
    fn check_neighboring_cells(&self) -> Result<(), SolvingError> {
        // Parcourt de toutes les cases de la grille avec un chiffre défini
        for (line_column, cell) in &self.grid.hashmap_cells {
            if let CellContent::Number(n) = cell.content {
                // Parcourt des cases voisines
                let neighboring_line_columns = NeighboringLineColumns::new(
                    *line_column,
                    self.grid.min_line_column,
                    self.grid.max_line_column,
                );
                for neighboring_line_column in neighboring_line_columns {
                    let option_cell = self.grid.get_cell(neighboring_line_column);
                    if let Some(neighboring_cell) = option_cell {
                        if let CellContent::Number(neighboring_n) = neighboring_cell.content {
                            // C'est une erreur si une case voisine contient le même chiffre
                            if n == neighboring_n {
                                return Err(SolvingError::NeighboringWithSameNumber(
                                    *line_column,
                                    neighboring_line_column,
                                    n,
                                ));
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Vérifie que pour toutes les cases définies d'une même zone, il n'y a pas un chiffre
    /// qui apparaît 2 fois
    fn check_zone_numbers(&self) -> Result<(), SolvingError> {
        // Parcourt de toutes les zones
        for (c_zone, zone) in &self.grid.hashmap_zones {
            // Init liste des chiffres définit dans la zone
            let mut zone_numbers = Simple09Set::default();
            // Parcourt des cases de la zone
            for line_column in &zone.set_line_column {
                let cell = match self.grid.get_cell(*line_column) {
                    None => return Err(SolvingError::BadImplementation),
                    Some(cell) => cell,
                };
                if let CellContent::Number(n) = cell.content {
                    // C'est une erreur si un même chiffre apparaît plusieurs fois dans la même zone
                    if zone_numbers.contains(n) {
                        return Err(SolvingError::ZoneWithSameNumber(*c_zone, n));
                    }
                    zone_numbers.insert(n);
                }
            }
        }
        Ok(())
    }

    /// Vérifie qu'il n'y a pas une case avec aucune valeur possible
    fn check_cell_with_no_possible_values(&self) -> Result<(), SolvingError> {
        // Parcourt de toutes les cases de la grille avec une liste de valeurs possibles
        for (line_column, cell) in &self.grid.hashmap_cells {
            if let CellContent::PossibleNumbers(hash_set) = &cell.content {
                if hash_set.is_empty() {
                    return Err(SolvingError::NoPossibleNumber(*line_column));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_check_ok() {
        let grid = Grid::from_str(
            "
        a1 b  b2
        b4 b  b
        c  c  c2
        ",
        )
        .unwrap();

        let solver = Solver::new(&grid);

        assert!(solver.check().is_ok());
    }

    #[test]
    fn test_check_voisin_nok() {
        let grid = Grid::from_str(
            "
        # NOK car a1 et b1 sont voisins
        a1 b  b2
        b4 b1 b
        c  c  c2
        ",
        )
        .unwrap();

        let solver = Solver::new(&grid);

        assert!(solver.check().is_err());
    }

    #[test]
    fn test_check_zone_nok() {
        let grid = Grid::from_str(
            "
        # NOK car 2 apparaît 2 x dans la zone b
        a1 b  b2
        b4 b2 b
        c  c  c2
        ",
        )
        .unwrap();

        let solver = Solver::new(&grid);

        assert!(solver.check().is_err());
    }

    #[test]
    fn test_is_solved_ok() {
        let grid = Grid::from_str(
            "
        a1 b3 b2
        b4 b5 b1
        c1 c3 c2
        ",
        )
        .unwrap();

        let solver = Solver::new(&grid);

        assert!(solver.check().is_ok());
        assert!(solver.is_solved());
    }

    #[test]
    fn test_is_solved_nok() {
        let grid = Grid::from_str(
            "
        a1 b  b2
        b4 b  b
        c  c  c2
        ",
        )
        .unwrap();

        let solver = Solver::new(&grid);

        assert!(solver.check().is_ok());
        assert!(!solver.is_solved());
    }
}
