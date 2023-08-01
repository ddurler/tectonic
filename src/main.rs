use std::str::FromStr;

use tectonic::{Grid, Solver};

pub fn main() {
    let mut grid = Grid::default();

    // Grille exemple dans le fichier README...

    // 1ere ligne (en utilisant la primitive 'add_cell')
    grid.add_cell((0, 0), 'a', Some(1));
    grid.add_cell((0, 1), 'b', None);
    grid.add_cell((0, 2), 'b', Some(2));

    // 2eme et 3eme lignes (en utilisant la primitive 'add_line')
    grid.add_line(1, vec![('b', Some(4)), ('b', None), ('b', None)]);
    grid.add_line(2, vec![('c', None), ('c', None), ('c', Some(2))]);

    // println!("Grid = {grid:?}");
    // println!();
    // println!("{grid}");

    // Autre façon de définir la grille
    let grid = Grid::from_str(
        "
    a1 b  b2
    b4 b  b
    c  c  c2
    ",
    )
    .unwrap();

    println!("{grid}");

    let mut solver = Solver::new(&grid);
    println!("{solver}");

    let _ = solver.solve();
}
