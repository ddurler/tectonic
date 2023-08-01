use std::collections::HashMap;
use std::collections::HashSet;
use std::error::Error;
use std::fmt;
use std::str::FromStr;

use crate::line_column::LineColumn;
// use crate::neighboring_line_columns::NeighboringLineColumns;

/// Information pour une zone de la grille tectonic
#[derive(Clone, Debug, Default)]
pub struct Zone {
    pub c_zone: char,
    pub set_line_column: HashSet<LineColumn>,
}

/// Contenu d'une case
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum CellContent {
    // Case avec un contenu non défini (construction initiale)
    #[default]
    Undefined,

    // Case avec un chiffre
    Number(u8),

    // Case avec une liste de chiffres possibles
    PossibleNumber(HashSet<u8>),
}

/// Information pour une case de la grille tectonic
#[derive(Clone, Debug, Default)]
pub struct Cell {
    pub c_zone: char,
    pub line_column: LineColumn,
    pub content: CellContent,
}

/// Représentation d'une grille tectonic
#[derive(Clone, Debug, Default)]
pub struct Grid {
    // Numéro de ligne/column min et max.
    pub min_line_column: LineColumn,
    pub max_line_column: LineColumn,

    // HashMap des différentes zones de la grille
    // La clef est la lettre utilisée lors de la construction pour désigner une zone
    pub hashmap_zones: HashMap<char, Zone>,

    // HashMap des différentes cases de la grille
    // La clef est la ligne_colonne de la case dans la grille
    pub hashmap_cells: HashMap<LineColumn, Cell>,
}

impl fmt::Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut res = String::new();
        // La première ligne contient les numéros de colonnes
        res.push_str("  ");
        for column in self.min_line_column.column..=self.max_line_column.column {
            res.push_str(&format!(" {column:2}"));
        }
        res.push('\n');
        // Contenu de la grille
        for line in self.min_line_column.line..=self.max_line_column.line {
            res.push_str(&format!("{line:2} ")); // Numéro de ligne à gauche
            for column in self.min_line_column.column..=self.max_line_column.column {
                let line_column = LineColumn::new(line, column);
                let option_cell = self.get_cell(line_column);
                let str_cell = match option_cell {
                    None => String::new(),
                    Some(cell) => {
                        let zone = cell.c_zone;
                        match cell.content {
                            CellContent::Undefined => format!("{zone}"),
                            CellContent::Number(n) => format!("{zone}{n}"),
                            CellContent::PossibleNumber(_) => format!("{zone}?"),
                        }
                    }
                };
                res.push_str(&format!("{str_cell:3}"));
            }
            res.push('\n');
        }

        write!(f, "{res}")
    }
}

impl Grid {
    /// Ajoute le contenu d'une case dans la grille tectonic en précisant
    /// `tuple_line_column` Coordonnées dans la grille où (0, 0) pourrait être le coin supérieur gauche
    /// `c_zone` représente une zone de la grille par une lettre
    /// `content` est le contenu de cette case qui peut être vide ou contenir déjà un chiffre
    pub fn add_cell(&mut self, tuple_line_column: (i32, i32), c_zone: char, content: Option<u8>) {
        let line_column = LineColumn::new(tuple_line_column.0, tuple_line_column.1);

        // Record min & max line/column
        self.min_line_column.min(line_column);
        self.max_line_column.max(line_column);

        let zone = self.get_or_create_zone(c_zone);
        zone.c_zone = c_zone;
        zone.set_line_column.insert(line_column);

        let cell = self.get_or_create_cell(line_column);
        cell.c_zone = c_zone;
        cell.line_column = line_column;
        cell.content = match content {
            None => CellContent::Undefined,
            Some(n) => CellContent::Number(n),
        };
    }

    /// Ajoute une ligne (à partir de la colonne 0) dans la grille tectonic en précisant
    /// `line` est un numéro de ligne (la 1ere ligne du haut est la ligne 0)
    /// Un tableau de couple (`c_zone`, `content`) où
    /// `c_zone` représente une zone de la grille par une lettre
    /// `content` est le contenu de cette case qui peut être vide ou contenir déjà un chiffre
    /// # Panics
    /// Cette fonction panics si le nombre de colonnes est excessif
    pub fn add_line(&mut self, line: i32, cells: Vec<(char, Option<u8>)>) {
        for (column, (c_zone, content)) in cells.into_iter().enumerate() {
            let column = i32::try_from(column).unwrap();
            self.add_cell((line, column), c_zone, content);
        }
    }

    /// Accesseur à une zone de la grille (créée si elle n'existe pas)
    #[must_use]
    fn get_or_create_zone(&mut self, c_zone: char) -> &mut Zone {
        self.hashmap_zones
            .entry(c_zone)
            .or_insert_with(Zone::default)
    }

    /// Accesseur à une zone de la grille (None) si elle n'existe pas
    #[must_use]
    pub fn get_zone(&mut self, c_zone: char) -> Option<&mut Zone> {
        self.hashmap_zones.get_mut(&c_zone)
    }

    /// Accesseur à une case de la grille (créée si elle n'existe pas)
    #[must_use]
    fn get_or_create_cell(&mut self, line_column: LineColumn) -> &mut Cell {
        self.hashmap_cells
            .entry(line_column)
            .or_insert_with(Cell::default)
    }

    /// Accesseur à une case non mutable de la grille (None) si elle n'existe pas
    #[must_use]
    pub fn get_cell(&self, line_column: LineColumn) -> Option<&Cell> {
        self.hashmap_cells.get(&line_column)
    }

    /// Accesseur à une case mutable de la grille (None) si elle n'existe pas
    #[must_use]
    pub fn get_mut_cell(&mut self, line_column: LineColumn) -> Option<&mut Cell> {
        self.hashmap_cells.get_mut(&line_column)
    }
}

/// Erreur rencontrée lors du parsing d'une grille avec `FromStr`
#[derive(Debug)]
pub struct ParseGridError(i32, i32);

impl fmt::Display for ParseGridError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Grid parsing error on line={}, column={}",
            self.0, self.1
        )
    }
}

impl Error for ParseGridError {}

impl FromStr for Grid {
    type Err = ParseGridError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut grid = Grid::default();

        // Numéro de ligne initialement
        let mut line = -1;

        for str_line in s.lines() {
            let str_line = str_line.trim();
            if !str_line.is_empty() {
                line += 1;

                // Numéro de colonne initialement
                let mut column = -1;

                for str_cell in str_line.split(' ') {
                    let vec_char: Vec<char> = str_cell.chars().collect();
                    if vec_char.is_empty() {
                        // Espace entre les espaces...
                        continue;
                    }

                    column += 1;

                    if vec_char.len() == 1 {
                        // Ne contient qu'un caractère pour la zone et pas de chiffre
                        let c_zone = vec_char[0];
                        grid.add_cell((line, column), c_zone, None);
                    } else if vec_char.len() == 2 {
                        // Contient un caractère pour la zone et un chiffre
                        let c_zone = vec_char[0];
                        let option_n = vec_char[1].to_digit(10);
                        match option_n {
                            None => return Err(ParseGridError(line, column)),
                            Some(n) => {
                                let n = u8::try_from(n).unwrap();
                                grid.add_cell((line, column), c_zone, Some(n));
                            }
                        }
                    } else {
                        // Définition incorrecte d'une case
                        return Err(ParseGridError(line, column));
                    }
                }
            }
        }

        Ok(grid)
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_grid_add_cell() {
        // Grille vierge
        let mut grid = Grid::default();

        // Ligne/Colonne/Zone/Number de la case placée
        let line_column = (1, 2);
        let c_zone = 'a';
        let content_number = 1;

        // Struct pour les positions ligne/colonne
        let struct_line_column = LineColumn::new(line_column.0, line_column.1);

        // Grille vierge ne connaît pas la zone ni la case
        assert!(grid.get_zone(c_zone).is_none());
        assert!(grid.get_cell(struct_line_column).is_none());

        // Ajoute la case qui contient la valeur dans la zone
        grid.add_cell(line_column, c_zone, Some(content_number));

        // Vérifie les dimensions de la grille en cours de construction
        // grid.min_line_column reste à (0,0)
        assert_eq!(grid.max_line_column.line, line_column.0);
        assert_eq!(grid.max_line_column.column, line_column.1);

        // Vérifie que la zone est maintenant connue
        assert!(grid.hashmap_zones.contains_key(&c_zone));
        assert!(grid.get_zone(c_zone).is_some());

        // Vérifie que la case placée est maintenant connue
        assert!(grid.hashmap_cells.contains_key(&struct_line_column));
        assert!(grid.get_cell(struct_line_column).is_some());

        // Vérifie que la case est bien référencée dans la zone
        let zone = grid.get_zone(c_zone).unwrap();
        assert_eq!(zone.c_zone, c_zone);
        assert!(zone.set_line_column.contains(&struct_line_column));

        // Vérifie que la case est correctement définie
        let cell = grid.get_cell(struct_line_column).unwrap();
        assert_eq!(cell.c_zone, c_zone);
        assert_eq!(cell.line_column, struct_line_column);
        assert_eq!(cell.content, CellContent::Number(content_number));
    }

    #[test]
    fn test_parse_grid_ok() {
        let result_grid = Grid::from_str(
            "
        a1 b  b2
        b4 b  b
        c  c  c2
        ",
        );

        assert!(result_grid.is_ok());
    }

    #[test]
    fn test_parse_grid_nok() {
        // NOK car une case b22 avec syntaxe incorrecte (line=1, column=1)
        let result_grid = Grid::from_str(
            "
        a1 b  b2
        b4 b22 b
        c  c  c2
        ",
        );

        assert!(result_grid.is_err());
        if let Err(ParseGridError(line, column)) = result_grid {
            assert_eq!(line, 1);
            assert_eq!(column, 1);
        } else {
            panic!("ParseGridError non détectée");
        }
    }

    #[test]
    fn test_parse_grid_nok_2() {
        // NOK car une case bz avec syntaxe incorrecte (line=1, column=1)
        let result_grid = Grid::from_str(
            "
        a1 b  b2
        b4 bz b
        c  c  c2
        ",
        );

        assert!(result_grid.is_err());
        if let Err(ParseGridError(line, column)) = result_grid {
            assert_eq!(line, 1);
            assert_eq!(column, 1);
        } else {
            panic!("ParseGridError non détectée");
        }
    }
}
