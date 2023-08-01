use std::collections::HashMap;

use crate::line_column::LineColumn;
// use crate::neighboring_line_columns::NeighboringLineColumns;

/// Information pour une zone de la grille tectonic
#[derive(Debug, Default)]
pub struct Zone {
    c_zone: char,
    vec_line_column: Vec<LineColumn>,
}

/// Information pour une case de la grille tectonic
#[derive(Debug, Default)]
pub struct Cell {
    c_zone: char,
    line_column: LineColumn,
    content: Option<u8>,
}

/// Représentation d'une grille tectonic
#[derive(Debug, Default)]
pub struct Grid {
    // Numéro de ligne/column min et max.
    min_line_column: LineColumn,
    max_line_column: LineColumn,

    // HashMap des différentes zones de la grille
    // La clef est la lettre utilisée lors de la construction pour désigner une zone
    hashmap_zones: HashMap<char, Zone>,

    // HashMap des différentes cases de la grille
    // La clef est la ligne_colonne de la case dans la grille
    hashmap_cells: HashMap<LineColumn, Cell>,
}

impl Grid {
    /// Ajoute le contenu d'une case dans la grille tectonic en précisant
    /// `tuple_line_column` Coordonnées dans la grille où (0, 0) pourrait être le coin supérieur gauche
    /// `c_zone` représente une zone de la grille par une lettre
    /// `content` est le contenu de cette case qui peut être vide ou contenir déjà un chiffre
    /// # Panics
    /// Cette fonction panic! si la case (`line`, `column`) est déjà définie
    pub fn add_cell(&mut self, tuple_line_column: (i32, i32), c_zone: char, content: Option<u8>) {
        let line_column = LineColumn::new(tuple_line_column.0, tuple_line_column.1);

        // Case déjà définie ?
        assert!(
            self.get_cell(line_column).is_none(),
            "La case ligne={} colonne={} est définie plusieurs fois",
            line_column.line,
            line_column.column
        );

        // Record min & max line/column
        self.min_line_column.min(line_column);
        self.max_line_column.max(line_column);

        let zone = self.get_or_create_zone(c_zone);
        zone.c_zone = c_zone;
        zone.vec_line_column.push(line_column);

        let cell = self.get_or_create_cell(line_column);
        cell.c_zone = c_zone;
        cell.line_column = line_column;
        cell.content = content;
    }

    /// Ajoute une ligne (à partir de la colonne 0) dans la grille tectonic en précisant
    /// `line` est un numéro de ligne (la 1ere ligne du haut est la ligne 0)
    /// Un tableau de couple (`c_zone`, `content`) où
    /// `c_zone` représente une zone de la grille par une lettre
    /// `content` est le contenu de cette case qui peut être vide ou contenir déjà un chiffre
    /// # Panics
    /// Cette fonction panics si une des cases de la ligne est déjà définie
    pub fn add_line(&mut self, line: i32, cells: Vec<(char, Option<u8>)>) {
        for (column, (c_zone, content)) in cells.into_iter().enumerate() {
            let column = i32::try_from(column).unwrap();
            self.add_cell((line, column), c_zone, content);
        }
    }

    /// Accesseur à une zone de la grille (créée si elle n'existe pas)
    fn get_or_create_zone(&mut self, c_zone: char) -> &mut Zone {
        self.hashmap_zones
            .entry(c_zone)
            .or_insert_with(Zone::default)
    }

    /// Accesseur à une zone de la grille (None) si elle n'existe pas
    #[allow(dead_code)]
    fn get_zone(&mut self, c_zone: char) -> Option<&mut Zone> {
        self.hashmap_zones.get_mut(&c_zone)
    }

    /// Accesseur à une case de la grille (créée si elle n'existe pas)
    fn get_or_create_cell(&mut self, line_column: LineColumn) -> &mut Cell {
        self.hashmap_cells
            .entry(line_column)
            .or_insert_with(Cell::default)
    }

    /// Accesseur à une case de la grille (None) si elle n'existe pas
    fn get_cell(&mut self, line_column: LineColumn) -> Option<&mut Cell> {
        self.hashmap_cells.get_mut(&line_column)
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_grid_add_cell() {
        // Grille vierge
        let mut grid = Grid::default();

        // Ligne/Colonne de la case placée
        let line_column = (1, 2);
        let c_zone = 'a';
        let content = Some(1);

        // Struct pour les positions ligne/colonne
        let struct_line_column = LineColumn::new(line_column.0, line_column.1);

        // Grille vierge ne connaît pas la zone ni la case
        assert!(grid.get_zone(c_zone).is_none());
        assert!(grid.get_cell(struct_line_column).is_none());

        // Ajoute la case qui contient la valeur dans la zone
        grid.add_cell(line_column, c_zone, content);

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
        assert!(zone.vec_line_column.contains(&struct_line_column));

        // Vérifie que la case est correctement définie
        let cell = grid.get_cell(struct_line_column).unwrap();
        assert_eq!(cell.c_zone, c_zone);
        assert_eq!(cell.line_column, struct_line_column);
        assert_eq!(cell.content, content);
    }
}
