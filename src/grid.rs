use std::collections::HashMap;

/// Information pour une zone de la grille tectonic
#[derive(Debug, Default)]
pub struct Zone {
    vec_line_column: Vec<(u8, u8)>,
}

/// Information pour une case de la grille tectonic
#[derive(Debug, Default)]
pub struct Cell {
    c_zone: char,
    content: Option<u8>,
}

/// Représentation d'une grille tectonic
#[derive(Debug, Default)]
pub struct Grid {
    // Numéro de ligne/column max.
    max_nb_line: u8,
    max_nb_column: u8,

    // HashMap des différentes zones de la grille
    // La clef est la lettre utilisée lors de la construction pour désigner une zone
    hashmap_zones: HashMap<char, Zone>,

    // HashMap des différentes cases de la grille
    // La clef est (ligne, colonne) de la case dans la grille
    hashmap_cells: HashMap<(u8, u8), Cell>,
}

impl Grid {
    /// Ajoute le contenu d'une case dans la grille tectonic en précisant
    /// (`line`, `column`) Coordonnées dans la case où (0, 0) est le coin supérieur gauche
    /// `c_zone` représente une zone de la grille par une lettre
    /// `content` est le contenu de cette case qui peut être vide ou contenir déjà un chiffre
    /// # Panics
    /// Cette fonction panic! si la case (`line`, `column`) est déjà définie
    pub fn add_cell(&mut self, line_column: (u8, u8), c_zone: char, content: Option<u8>) {
        // Case déjà définie ?
        assert!(
            self.get_cell(line_column).is_none(),
            "La case ligne={} colonne={} est définie plusieurs fois",
            line_column.0,
            line_column.1
        );

        self.max_nb_line = u8::max(self.max_nb_line, line_column.0);
        self.max_nb_column = u8::max(self.max_nb_column, line_column.1);

        let zone = self.get_or_create_zone(c_zone);
        zone.vec_line_column.push(line_column);

        let cell = self.get_or_create_cell(line_column);
        cell.c_zone = c_zone;
        cell.content = content;
    }

    /// Ajoute une ligne dans la grille tectonic en précisant
    /// `line` est un numéro de ligne (la 1ere ligne du haut est la ligne 0)
    /// Un tableau de couple (`c_zone`, `content`) où
    /// `c_zone` représente une zone de la grille par une lettre
    /// `content` est le contenu de cette case qui peut être vide ou contenir déjà un chiffre
    /// # Panics
    /// Cette fonction panics si une des cases de la ligne est déjà définie
    pub fn add_line(&mut self, line: u8, cells: Vec<(char, Option<u8>)>) {
        for (column, (c_zone, content)) in cells.into_iter().enumerate() {
            let column = u8::try_from(column).expect("Max 255 cases par ligne");
            self.add_cell((line, column), c_zone, content);
        }
    }

    /// Accesseur à une zone de la grille (créée si elle n'existe pas)
    fn get_or_create_zone(&mut self, c_zone: char) -> &mut Zone {
        self.hashmap_zones
            .entry(c_zone)
            .or_insert_with(Zone::default)
    }

    /// Assesseur à une zone de la grille (None) si elle n'existe pas
    fn get_zone(&mut self, c_zone: char) -> Option<&mut Zone> {
        self.hashmap_zones.get_mut(&c_zone)
    }

    /// Accesseur à une case de la grille (créée si elle n'existe pas)
    fn get_or_create_cell(&mut self, line_column: (u8, u8)) -> &mut Cell {
        self.hashmap_cells
            .entry(line_column)
            .or_insert_with(Cell::default)
    }

    /// Assesseur à une case de la grille (None) si elle n'existe pas
    fn get_cell(&mut self, line_column: (u8, u8)) -> Option<&mut Cell> {
        self.hashmap_cells.get_mut(&line_column)
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_add_cell() {
        // Grille vierge
        let mut grid = Grid::default();

        // Grille vierge ne connaît pas la zone 'a' ni la case en (1, 2)
        assert!(grid.get_zone('a').is_none());
        assert!(grid.get_cell((1, 2)).is_none());

        // Ajoute la case (1, 2) qui contient la valeur 1 dans la zone 'a'
        grid.add_cell((1, 2), 'a', Some(1));

        // Vérifie les dimensions de la grille en cours de construction
        assert_eq!(grid.max_nb_line, 1);
        assert_eq!(grid.max_nb_column, 2);

        // Vérifie que la zone 'a' est maintenant connue
        assert!(grid.hashmap_zones.contains_key(&'a'));
        assert!(grid.get_zone('a').is_some());

        // Vérifie que la case en (1, 2) est maintenant connue
        assert!(grid.hashmap_cells.contains_key(&(1, 2)));
        assert!(grid.get_cell((1, 2)).is_some());

        // Vérifie que la case (1, 2) est bien référencée dans la zone 'a'
        let zone = grid.get_zone('a').unwrap();
        assert!(zone.vec_line_column.contains(&(1, 2)));

        // Vérifie que la case (1, 2) est correctement définie
        let cell = grid.get_cell((1, 2)).unwrap();
        assert_eq!(cell.c_zone, 'a');
        assert_eq!(cell.content, Some(1));
    }
}
