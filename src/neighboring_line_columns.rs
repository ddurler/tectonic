use crate::line_column::LineColumn;

/// Positions voisines d'une case
///
/// Cette structure permet d'itérer sur toutes les cases voisines dans la grille.
///
/// Une case est voisine dans toutes les directions (y compris dans les diagonales).
/// La taille de la grille (min et max) pour les lignes et les colonnes est spécifiées pour ne pas
/// faire apparaître de case hors de la grille lors de l'itération.
#[derive(Debug)]
pub struct NeighboringLineColumns {
    line_column: LineColumn,
    min_line_column: LineColumn,
    max_line_column: LineColumn,
    yield_directions: Vec<(i32, i32)>,
}

impl NeighboringLineColumns {
    #[allow(dead_code)]
    pub fn new(
        line_column: LineColumn,
        min_line_column: LineColumn,
        max_line_column: LineColumn,
    ) -> Self {
        NeighboringLineColumns {
            line_column,
            min_line_column,
            max_line_column,
            yield_directions: Vec::new(),
        }
    }
}

impl Iterator for NeighboringLineColumns {
    type Item = LineColumn;

    fn next(&mut self) -> Option<Self::Item> {
        // Toutes les directions possibles autour de la case
        let directions: Vec<(i32, i32)> = vec![
            (-1, 0),
            (-1, 1),
            (0, 1),
            (1, 1),
            (1, 0),
            (1, -1),
            (0, -1),
            (-1, -1),
        ];

        // On parcourt toutes les directions non encore étudiées
        for direction in directions {
            if !self.yield_directions.contains(&direction) {
                // Direction qui sera maintenant étudiée
                self.yield_directions.push(direction);
                // Case existante ?
                let neighboring_line = self.line_column.line + direction.0;
                if neighboring_line >= self.min_line_column.line
                    && neighboring_line <= self.max_line_column.line
                {
                    let neighboring_column = self.line_column.column + direction.1;
                    if neighboring_column >= self.min_line_column.column
                        && neighboring_column <= self.max_line_column.column
                    {
                        // Case possible, on retourne cette case
                        return Some(LineColumn::new(neighboring_line, neighboring_column));
                    }
                }
            }
        }

        // Plus de case voisine...
        None
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_neighboring_cells() {
        // La grille est limité à (0, 0) - (5, 5)
        let min_line_column = (0, 0);
        let max_line_column = (5, 5);

        // Ce jeu de tests définit la case centrale et la liste des cases voisines dans la grille
        let vec_tests = vec![
            (
                LineColumn::new(1, 1),
                vec![
                    LineColumn::new(0, 0),
                    LineColumn::new(0, 1),
                    LineColumn::new(0, 2),
                    LineColumn::new(1, 0),
                    LineColumn::new(1, 2),
                    LineColumn::new(2, 0),
                    LineColumn::new(2, 1),
                    LineColumn::new(2, 2),
                ],
            ),
            (
                LineColumn::new(0, 0),
                vec![
                    LineColumn::new(0, 1),
                    LineColumn::new(1, 0),
                    LineColumn::new(1, 1),
                ],
            ),
            (
                LineColumn::new(5, 5),
                vec![
                    LineColumn::new(4, 4),
                    LineColumn::new(4, 5),
                    LineColumn::new(5, 4),
                ],
            ),
        ];

        let min_line_column = LineColumn::new(min_line_column.0, min_line_column.1);
        let max_line_column = LineColumn::new(max_line_column.0, max_line_column.1);

        for test in vec_tests {
            // Iterator de toutes les cases voisines
            let neighboring_cells =
                NeighboringLineColumns::new(test.0, min_line_column, max_line_column);
            let neighboring_cells_found: Vec<LineColumn> = neighboring_cells.into_iter().collect();

            // assert_eq!(neighboring_cells_found, test.1) ?
            for v in test.1 {
                assert!(neighboring_cells_found.contains(&v));
            }
        }
    }
}
