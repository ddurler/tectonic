use std::fmt;

use crate::grid::{CellContent, Grid};
use crate::line_column::LineColumn;
use crate::neighboring_line_columns::NeighboringLineColumns;

/// Cas d'erreurs possibles pendant la résolution de la grille tectonic
#[derive(Debug)]
pub enum SolvingError {
    NeighboringWithSameNumber(LineColumn, LineColumn, u8),
    ZoneWithSameNumber(char, u8),

    // Erreur d'implémentation qui ne devrait pas arriver :)
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
            SolvingError::BadImplementation => write!(f, "Erreur inattendue (voir source code...)"),
        }
    }
}

impl std::error::Error for SolvingError {}

/// Étapes de la résolution d'une grille tectonic
#[derive(Debug, Default)]
enum SolvingStep {
    // Solver qui vient d'être créé
    #[default]
    Newer,
}
/// Structure pour la résolution d'une grille tectonic
#[derive(Debug, Default)]
pub struct Solver {
    grid: Grid,
    _solving_step: SolvingStep,
}

impl fmt::Display for Solver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.grid)
    }
}

impl Solver {
    #[must_use]
    pub fn new(grid: &Grid) -> Self {
        Solver {
            grid: grid.clone(),
            _solving_step: SolvingStep::Newer,
        }
    }

    /// Applique une étape de résolution
    /// Retourne true si la grille est résolue, false sinon
    /// # Errors
    /// Une erreur est retournée si la grille n'est pas (ou plus) cohérente
    pub fn solve(&mut self) -> Result<bool, SolvingError> {
        self.check()?;

        Ok(false) // TODO
    }

    /// Vérifie la consistance de la grille
    fn check(&self) -> Result<(), SolvingError> {
        self.check_neighboring_cells()?;
        self.check_zone_numbers()?;
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
            let mut vec_numbers = Vec::new();
            // Parcourt des cases de la zone
            for line_column in &zone.set_line_column {
                let cell = match self.grid.get_cell(*line_column) {
                    None => return Err(SolvingError::BadImplementation),
                    Some(cell) => cell,
                };
                if let CellContent::Number(n) = cell.content {
                    // C'est une erreur si un même chiffre apparaît plusieurs fois dans la même zone
                    if vec_numbers.contains(&n) {
                        return Err(SolvingError::ZoneWithSameNumber(*c_zone, n));
                    }
                    vec_numbers.push(n);
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
        // NOK car a1 et b1 sont voisins
        let grid = Grid::from_str(
            "
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
        // NOK car 2 apparaît 2 x dans la zone b
        let grid = Grid::from_str(
            "
        a1 b  b2
        b4 b2 b
        c  c  c2
        ",
        )
        .unwrap();

        let solver = Solver::new(&grid);

        assert!(solver.check().is_err());
    }
}
