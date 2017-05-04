use std::collections::HashMap;
use std::collections::VecDeque;


/* Board is serialized as one i64 created from player whose turn it is (2 bits) & 9 Squares (each 6 bits) in this order:
1 2 3
8 0 4
7 6 5
This order is selected to make board rotation easy which is important for normalising (to avoid
counting same boards multiple times)
*/


type Size = u8; // SMALL, MEDIUM, LARGE
type Color = u8; // EMPTY, ORANGE, BLUE
type Figure = u8; // Size * Color
type Square = u8; // sum of containing figures
type Position = u8;
type Board = u64;

const SMALL: Size = 1;
const MEDIUM: Size = 4;
const LARGE: Size = 16;

const ORANGE: Color = 1;
const BLUE: Color = 2;

const EMPTY: u8 = 0;

const SQUARE_LENGTH: u8 = 6;
const SQUARE_MASK: u64 = 63;

fn get_initial_board() -> Board {
    let mut board = 0;

    board = set_board_square(board, 0, LARGE * ORANGE);
    board = set_board_square(board, 5, SMALL * BLUE + MEDIUM * ORANGE + LARGE * BLUE);
    board = set_board_square(board, 2, SMALL * ORANGE + MEDIUM * BLUE);
    board = set_board_square(board, 8, LARGE * BLUE);
    board = set_board_square(board, 4, MEDIUM * ORANGE);
    board = set_board_square(board, 7, LARGE * ORANGE);
    board = set_board_square(board, 3, MEDIUM * BLUE);

    board = set_board_player(board, ORANGE);
    board = normalize_board(board);

    board
}

macro_rules! hashmap {
    ($( $key: expr => $val: expr ),*) => {{
         let mut map = ::std::collections::HashMap::new();
         $( map.insert($key, $val); )*
         map
    }}
}


fn get_board_square(board: Board, position: Position) -> Square {
    let shift = position * SQUARE_LENGTH;
    ((board >> shift) & SQUARE_MASK) as Square
}

fn set_board_square(board: Board, position: Position, square: Square) -> Board {
    let shift = position * SQUARE_LENGTH;
    let reset_board = board & !(SQUARE_MASK << shift);
    let board_with_item = (square as Board) << shift;
    reset_board | board_with_item
}

fn get_board_player(board: Board) -> Color {
    (board >> (SQUARE_LENGTH * 9)) as Color
}

fn set_board_player(board: Board, color: Color) -> Board {
    let shift = SQUARE_LENGTH * 9;
    let reset_mask = !(3 << shift);
    (board & reset_mask) | ((color as u64) << shift)
}

fn get_board_remaining(board: Board) -> Vec<Size> {
    let playing = get_board_player(board);

    let mut small = 2;
    let mut medium = 2;
    let mut large = 2;

    for position in 0..9 {
        let item = get_board_square(board, position);
        if item & (playing * SMALL) != 0 { small -= 1; }
        if item & (playing * MEDIUM) != 0 { medium -= 1; }
        if item & (playing * LARGE) != 0 { large -= 1; }
    }

    let mut remaining = Vec::new();
    if small > 0 {
        remaining.push(SMALL);
    };
    if medium > 0 {
        remaining.push(MEDIUM);
    };
    if large > 0 {
        remaining.push(LARGE);
    };

    remaining
}

fn get_figure_size(figure: Figure) -> Size {
    match figure {
        1 | 2 => SMALL,
        4 | 8 => MEDIUM,
        16 | 32 => LARGE,
        _ => panic!("Invalid figure size {}", figure),
    }
}

fn get_opponent(player: Color) -> Color {
    match player {
        ORANGE => BLUE,
        BLUE => ORANGE,
        _ => panic!("Invalid player color {}", player),
    }
}

fn get_color(square: Square, size: Size) -> Color {
    let shift = get_size_shift(size);
    (square >> shift) & 3 as Color
}

fn get_size_shift(size: Size) -> u8 {
    match size {
        SMALL => 0,
        MEDIUM => 2,
        LARGE => 4,
        _ => panic!(),
    }
}

fn can_play_at(square: Square, size: Size) -> bool {
    let shift = get_size_shift(size);
    (square >> shift) == EMPTY
}

fn get_square_figure(square: Square, size: u8) -> u8 {
    let shift = get_size_shift(size);
    ((square >> shift) & 3) << shift
}

fn get_top_figure(square: Square) -> Color {
    for &size in &[LARGE, MEDIUM, SMALL] {
        let figure = get_square_figure(square, size);
        if figure != EMPTY {
            return figure;
        }
    }
    EMPTY
}

fn get_top_color(board: Board, position: Position) -> Color {
    let square = get_board_square(board, position);

    for &size in &[LARGE, MEDIUM, SMALL] {
        let color = get_color(square, size);
        if color != EMPTY {
            return color;
        }
    }

    EMPTY
}

fn is_triple(board: Board, positions: &[u8]) -> Color {
    let col1 = get_top_color(board, positions[0]);
    let col2 = get_top_color(board, positions[1]);
    let col3 = get_top_color(board, positions[2]);

    if (col1 == col2) && (col2 == col3) {
        return col1;
    } else {
        return EMPTY;
    }
}

fn get_winner(board: Board) -> Color {
    let triples = [
        [1, 2, 3], [8, 0, 4], [7, 6, 5],
        [1, 8, 7], [2, 0, 6], [3, 4, 5],
        [1, 0, 5], [3, 0, 7]
    ];

    for triple in triples.iter() {
        let winner = is_triple(board, triple);
        if winner != EMPTY {
            return winner;
        }
    }

    EMPTY
}

fn rotate_board(board: Board) -> Board {
    let player = get_board_player(board);
    let zero = board & SQUARE_MASK;
    let one_two_mask = (SQUARE_MASK << SQUARE_LENGTH) | (SQUARE_MASK << (SQUARE_LENGTH * 2));
    let seven_mask = SQUARE_MASK << (SQUARE_LENGTH * 7);
    let one_two = (board & one_two_mask) << (SQUARE_LENGTH * 6);
    let shifted_board = board >> (SQUARE_LENGTH * 2) & !SQUARE_MASK & !seven_mask;
    set_board_player(shifted_board | one_two | zero, player)
}

fn normalize_board(board: Board) -> Board {
    let mut min_board = board;
    let mut rotated_board = board;
    for _ in 0..3 {
        rotated_board = rotate_board(rotated_board);
        if rotated_board < min_board {
            min_board = rotated_board;
        }
    }
    min_board
}

fn modify_parents(parents: Vec<u64>, parent_is_winner: bool, visited: & mut HashMap<Board, (Vec<Board>, u8, i8)>) -> Vec<u64> {
    let mut next_parents: Vec<u64> = vec!();
    for key in parents {
        let new_state: (Vec<u64>, u8, i8);
        {
            let state = visited.get(&key).expect(&*format!("found parent that hasn't its own record: {}", key));
            if state.2 == -1 {
                continue; // parent already knows how to win
            }
            if parent_is_winner {
                new_state = (state.0.to_vec(), state.1, -1);
            } else {
                new_state = (state.0.to_vec(), state.1, state.2 + 1);
            }
        }
        if new_state.2 == -1 || (new_state.1 as i8) == new_state.2 {
            let mut temp = new_state.0.to_vec();
            next_parents.append(&mut temp);
        }
        visited.insert(key, new_state);
    }
    next_parents
}

fn add_new_items(board: Board, playing: Color, opponent: Color, to_visit: & mut VecDeque<(Board, Option<Board>)>) -> u8 {
    let mut num_of_children = 0;
    let remaining = get_board_remaining(board);
    for size in remaining {
        for pos in 0..9 {
            let square = get_board_square(board, pos);
            if !can_play_at(square, size) {
                continue;
            }
            let new_board = set_board_square(board, pos, square + playing * size);
            let new_normalized_board = normalize_board(new_board);
            let new_board_with_player = set_board_player(new_normalized_board, opponent);
            to_visit.push_back((new_board_with_player, Some(board)));
            num_of_children += 1;
        }
    }
    num_of_children
}

fn add_moved_items(board: Board, playing: Color, opponent: Color, to_visit: & mut VecDeque<(Board, Option<Board>)>) -> u8 {
    let mut num_of_children = 0;
    for pos in 0..9 {
        let color = get_top_color(board, pos);
        if color != playing {
            continue;
        }
        let square = get_board_square(board, pos);
        let figure = get_top_figure(square);
        let size = get_figure_size(figure);
        let board_without_figure = set_board_square(board, pos, square - figure);
        if get_winner(board_without_figure) != EMPTY {
            continue;
        }

        for target_pos in 0..9 {
            if target_pos == pos {
                continue;
            }
            let square = get_board_square(board_without_figure, target_pos);
            if !can_play_at(square, size) {
                continue;
            }

            let new_board = set_board_square(board_without_figure, target_pos, square + figure);
            let new_normalized_board = normalize_board(new_board);
            let new_board_with_player = set_board_player(new_normalized_board, opponent);
            to_visit.push_back((new_board_with_player, Some(board)));
            num_of_children += 1;
        }
    }
    num_of_children
}

fn is_first_state_better(first: (u8, i8), second: (u8, i8)) -> bool {
    if second.0 == 0 {
        return false;
    } else if first.0 == 0 {
        return true;
    } else if second.1 == -1 {
        return true;
    } else if first.1 == -1 {
        return false;
    } else if first.1 > second.1 {
        return true;
    } else if first.1 < second.1 {
        return false;
    } else {
        return first.0 < second.0;
    }
}

fn find_best_move(board: Board, visited: & HashMap<Board, (Vec<Board>, u8, i8)>) -> Option<(Board, u8, i8)> {
    let mut best: Option<(Board, u8, i8)> = None;

    for (&visited_board, state) in visited {
        if state.0.contains(&board) {
            let is_better: bool = if best.is_none() {
                true
            } else {
                let unwrap = best.unwrap();
                is_first_state_better((state.1, state.2), (unwrap.1, unwrap.2))
            };

            if is_better {
                best = Some((visited_board, state.1, state.2));
            }
        }
    }

    best
}

fn print_board(board: Board) {
    print_item(get_board_square(board, 1));
    print!(" ");
    print_item(get_board_square(board, 2));
    print!(" ");
    print_item(get_board_square(board, 3));
    println!();
    print_item(get_board_square(board, 8));
    print!(" ");
    print_item(get_board_square(board, 0));
    print!(" ");
    print_item(get_board_square(board, 4));
    println!();
    print_item(get_board_square(board, 7));
    print!(" ");
    print_item(get_board_square(board, 6));
    print!(" ");
    print_item(get_board_square(board, 5));
    println!("\nPlaying: {}\n\n", if get_board_player(board) == ORANGE {"ORANGE"} else {"BLUE"});
}

fn print_item(item: Square) {
    let mut res: String = "".to_string();
    for &size in &[LARGE, MEDIUM, SMALL] {
        res += match get_color(item, size) {
            ORANGE => "O",
            BLUE => "B",
            _ => ".",
        };
    }

    print!("{}", res);
}

fn main() {
//    print_board(36030033969680385);
//    return;

    // board -> (parents, num_of_children, number of moves from here which leads to lose (-1 when there is move which leads to win))
    let mut visited: HashMap<Board, (Vec<Board>, u8, i8)> = HashMap::new();

    // board, parent
    let mut to_visit: VecDeque<(Board, Option<Board>)> = VecDeque::new();

    let initial_board = get_initial_board();
    to_visit.push_back((initial_board, None));

    println!("Initial board:");
    print_board(initial_board);

    let mut winners = 0;
    let mut same = 0;
    for _ in 0..100_000_000 {
        let state_option = to_visit.pop_front();
        if state_option.is_none() {
            println!("all is done");
            break;
        }

        let (board, parent) = state_option.unwrap();
        let playing = get_board_player(board);
        let opponent = get_opponent(playing);

        if visited.contains_key(&board) {
            // I've already been in this state => just add parent and go for next state
            same += 1;
            let new_state: (Vec<u64>, u8, i8);
            {
                let state = visited.get(&board).unwrap();
                let mut new_parents = state.0.to_vec();
                if parent.is_some() {
                    new_parents.push(parent.unwrap());
                }

                new_state = (new_parents, state.1, state.2);
            }
            visited.insert(board, new_state);

            continue;
        }

        let winner = get_winner(board);
        if winner != EMPTY {
            assert_ne!(winner, playing);
            winners += 1;

            let mut parents = if parent.is_some() {vec!(parent.unwrap())} else {vec!()};

            visited.insert(board, (parents.to_vec(), 0, 0));

            let mut parent_is_winner = true;
            while !parents.is_empty() {
                parents = modify_parents(parents, parent_is_winner, & mut visited);
                parent_is_winner = !parent_is_winner;
            }

            continue;
        }

        let mut num_of_children = add_new_items(board, playing, opponent, & mut to_visit);
        num_of_children += add_moved_items(board, playing, opponent, & mut to_visit);

        let parents = if parent.is_some() {vec![parent.unwrap()]} else {vec!()};
        visited.insert(board, (parents, num_of_children, 0));
    }

    println!("Visited: {}", visited.len());
    println!("Winners: {}", winners);
    println!("Same: {}", same);
    println!("To visit: {}", to_visit.len());
    println!();

//    if let Some(state) = visited.get(&initial_board) {
//        println!("Info: {} -> {:?}", initial_board, state);
//    }

    let best_board = find_best_move(initial_board, &visited);
    if best_board.is_some() {
        println!("Best move:");
        let best_board = best_board.unwrap();

        print_board(best_board.0);
        println!("Info: {:?}", best_board);
    } else {
        println!("No boards are computed.");
    }

//    for (board, info) in &visited {
//        if info.0.contains(&initial_board) {
//            println!("Status: {} / {}", info.2, info.1);
//            print_board(*board);
//        }
//    }
}




#[cfg(test)]
mod tests {
    use super::*;

    fn get_default_board() -> Board {
        let mut board = 0;
        board = set_board_square(board, 2, SMALL * ORANGE + LARGE * BLUE);
        board = set_board_square(board, 6, SMALL * BLUE + LARGE * ORANGE);
        board = set_board_square(board, 0, MEDIUM * ORANGE + LARGE * BLUE);
        board = set_board_square(board, 1, SMALL * BLUE + LARGE * ORANGE);
        board = set_board_player(board, ORANGE);

        board
    }

    #[test]
    fn test_get_board_square() {
        let board = get_default_board();
        assert_eq!(get_board_square(board, 0), MEDIUM * ORANGE + LARGE * BLUE);
        assert_eq!(get_board_square(board, 3), EMPTY);
        assert_eq!(get_board_square(board, 6), SMALL * BLUE + LARGE * ORANGE);
    }

    #[test]
    fn test_set_board_square() {
        let mut board = 0;
        board = set_board_square(board, 2, SMALL * ORANGE + LARGE * BLUE);
        board = set_board_square(board, 2, MEDIUM * BLUE);
        assert_eq!(get_board_square(board, 2), MEDIUM * BLUE);
    }

    #[test]
    fn test_get_board_remaining() {
        let board = get_default_board();
        assert_eq!(get_board_remaining(board), vec![SMALL, MEDIUM]);

        let board = set_board_square(board, 3, SMALL * ORANGE);
        assert_eq!(get_board_remaining(board), vec![MEDIUM]);

        let board = set_board_square(board, 8, SMALL * ORANGE + MEDIUM * ORANGE);
        assert_eq!(get_board_remaining(board), vec![]);

        let board = set_board_player(0, ORANGE);
        assert_eq!(get_board_remaining(board), vec![SMALL, MEDIUM, LARGE]);
    }

    #[test]
    fn test_get_figure_size() {
        assert_eq!(get_figure_size(SMALL * ORANGE), SMALL);
        assert_eq!(get_figure_size(SMALL * BLUE), SMALL);
        assert_eq!(get_figure_size(LARGE * BLUE), LARGE);
    }

    #[test]
    fn test_get_color() {
        assert_eq!(get_color(SMALL * ORANGE + MEDIUM * BLUE, SMALL), ORANGE);
        assert_eq!(get_color(SMALL * ORANGE + MEDIUM * BLUE, MEDIUM), BLUE);
        assert_eq!(get_color(SMALL * ORANGE + MEDIUM * BLUE, LARGE), EMPTY);
        assert_eq!(get_color(EMPTY, SMALL), EMPTY);
        assert_eq!(get_color(EMPTY, LARGE), EMPTY);
        assert_eq!(get_color(LARGE * ORANGE, SMALL), EMPTY);
        assert_eq!(get_color(LARGE * ORANGE, LARGE), ORANGE);
    }

    #[test]
    fn test_can_play_at() {
        assert_eq!(can_play_at(SMALL * ORANGE + MEDIUM * BLUE, SMALL), false);
        assert_eq!(can_play_at(SMALL * ORANGE + MEDIUM * BLUE, MEDIUM), false);
        assert_eq!(can_play_at(SMALL * ORANGE + MEDIUM * BLUE, LARGE), true);
        assert_eq!(can_play_at(EMPTY, SMALL), true);
        assert_eq!(can_play_at(EMPTY, LARGE), true);
        assert_eq!(can_play_at(LARGE * ORANGE, SMALL), false);
        assert_eq!(can_play_at(LARGE * ORANGE, LARGE), false);
    }

    #[test]
    fn test_get_square_figure() {
        assert_eq!(get_square_figure(SMALL * ORANGE + MEDIUM * BLUE, SMALL), SMALL * ORANGE);
        assert_eq!(get_square_figure(SMALL * ORANGE + MEDIUM * BLUE, MEDIUM), MEDIUM * BLUE);
        assert_eq!(get_square_figure(SMALL * ORANGE + MEDIUM * BLUE, LARGE), EMPTY);
        assert_eq!(get_square_figure(EMPTY, SMALL), EMPTY);
        assert_eq!(get_square_figure(EMPTY, LARGE), EMPTY);
        assert_eq!(get_square_figure(LARGE * ORANGE, SMALL), EMPTY);
        assert_eq!(get_square_figure(LARGE * ORANGE, LARGE), LARGE * ORANGE);
    }

    #[test]
    fn test_get_top_figure() {
        assert_eq!(get_top_figure(SMALL * ORANGE + MEDIUM * BLUE), MEDIUM * BLUE);
        assert_eq!(get_top_figure(EMPTY), EMPTY);
        assert_eq!(get_top_figure(LARGE * ORANGE), LARGE * ORANGE);
    }

    #[test]
    fn test_rotate_board() {
        let board = get_default_board();
        let board1 = rotate_board(board);
        let board2 = rotate_board(board1);
        let board3 = rotate_board(board2);
        let board4 = rotate_board(board3);
        assert_ne!(board, board1);
        assert_ne!(board1, board2);
        assert_ne!(board2, board3);
        assert_ne!(board3, board4);
        assert_eq!(board, board4);
    }
}
