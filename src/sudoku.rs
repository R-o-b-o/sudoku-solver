use std::fmt;
use std::char;

#[derive(Clone)]
pub struct Sudoku {
    board: [char; 81],
    possible: [u32; 81],
}

impl Sudoku {
    pub fn new() -> Sudoku {
        Sudoku {
            board: ['.'; 81],
            possible: [0b0111111111u32;81],
        }
    }

    /// Initialise sudoku from puzzle string
    pub fn from_str(board_str: String) -> Result<Sudoku, &'static str> {
        let mut board_vec: Vec<char> = board_str.chars().collect();
        let mut board_array = ['.'; 81];

        if board_str.len() < 81 { return Err("input too short"); }
        if board_str.len() > 81 { return Err("input too long"); }
        
        
        for i in (0..board_array.len()).rev() {
            board_array[i] = board_vec.pop().unwrap();
        }

        let mut sudoku = Sudoku {
            board: board_array,
            possible: [0;81],
        };

        sudoku.update_possible();

        if sudoku.is_valid() {
            Ok(sudoku)
        } else {
            Err("invalid sudoku")
        }
    }

    pub fn to_str(&self) -> String {
        self.board.iter().collect()
    }

    pub fn update_cell(&mut self, index: usize, value: char) {
        self.board[index] = value;
        self.update_possible();
    }

    pub fn solve(&self) -> Option<Sudoku> {
        let mut sudoku_guess = self.clone();

        sudoku_guess.solve_naked_pairs();
        
        let mut guess_indices: Vec<usize> = Vec::new();

        for i in 0..sudoku_guess.board.len() {
            if sudoku_guess.board[i] == '.' {
                guess_indices.push(i);
            }
        } 
        
        sudoku_guess.solve_naked_singles(&mut guess_indices);
        sudoku_guess.solve_hidden_singles(&mut guess_indices);
        
        let mut guess_choices = 11;
        let mut guess_index = sudoku_guess.board.len() + 1;

        for i in guess_indices {
            let temp_choices = sudoku_guess.possible[i].count_ones();

            if temp_choices < guess_choices {
                guess_index = i;
                guess_choices = temp_choices;
            }
        }
        
        
        if guess_index == sudoku_guess.board.len() + 1 {
            Some(sudoku_guess)
        } else {
            let index = guess_index;
            let mut guesses = sudoku_guess.possible_to_vec(sudoku_guess.possible[index]);
    
            while guesses.len() > 0 {
                
                let guess = guesses.pop().unwrap();

                let mut sud_temp = sudoku_guess.clone();

                sud_temp.fill_blank_cell(index, guess);

                let sudoku_sol = sud_temp.solve();
                
                if sudoku_sol.is_some() {
                    return Some(sudoku_sol.unwrap());
                }
            }

            None
        }
    }

    /// Get u32 containing possible values left for the chunk 
    fn get_possible_from_chunk(&self, chunk: [char;9]) -> u32 {
        let mut cell_possible = 0b0111111111u32;

        for &cell in chunk.iter() {
            if cell != '.' {
                cell_possible &= !(1<<(cell.to_digit(10).unwrap()-1));
            }
        }

        cell_possible
    }

    // Get all 9 cell chunks from board (rows, columns, squares)
    fn get_chunks(&self) -> Vec<[usize; 9]> {
        let mut chunks: Vec<[usize; 9]> = Vec::new();

        let mut chunk_temp: [usize; 9] = Default::default();
        let mut chunk_temp_index;

        // fill rows
        for row_start in (0..81).step_by(9) {
            for row_offset in 0..9 {
                chunk_temp[row_offset] = row_start + row_offset;
            }
            chunks.push(chunk_temp);
        }

        // fill columns
        for col_start in 0..9 {
            chunk_temp_index = 0;
            for col_offset in (0..81).step_by(9) {
                chunk_temp[chunk_temp_index] = col_start+col_offset;
                chunk_temp_index += 1;
            }
            chunks.push(chunk_temp);
        }

        // fill squares
        for grid_start in [0, 3, 6, 27, 30, 33, 54, 57, 60].iter(){ // could also do for i in 0..27 + for j in 0..3
            chunk_temp_index = 0;
            for grid_offset1 in (0..19).step_by(9) {
                for grid_offset2 in 0..3 {
                    chunk_temp[chunk_temp_index] = grid_start + grid_offset1 + grid_offset2;
                    chunk_temp_index += 1;
                }
            }

            chunks.push(chunk_temp);
        }

        chunks
    }

    /// Get all 9 cell chunks which contain the cell_index cell
    fn get_chunks_containing_cell(&self, cell_index: usize) -> Vec<[usize; 9]>  {
        let mut chunks: Vec<[usize; 9]> = Vec::new();

        let mut chunk_temp: [usize; 9] = Default::default();
        let mut chunk_temp_index: usize;

        // find row
        let row_start = (cell_index / 9) * 9;
        for row_offset in 0..9 {
            chunk_temp[row_offset] = row_start + row_offset;
        }
        chunks.push(chunk_temp);

        // find column
        let col_start = cell_index % 9;
        chunk_temp_index = 0;
        for col_offset in (0..81).step_by(9) {
            chunk_temp[chunk_temp_index] = col_start+col_offset;
            chunk_temp_index += 1;
        }
        chunks.push(chunk_temp);

        
        // find square
        let grid_start = (cell_index / 27) * 27 + ((cell_index % 9) / 3) * 3;
        chunk_temp_index = 0;
        for grid_offset1 in (0..19).step_by(9) {
            for grid_offset2 in 0..3 {
                chunk_temp[chunk_temp_index] = grid_start + grid_offset1 + grid_offset2;
                chunk_temp_index += 1;
            }
        }
        
        chunks.push(chunk_temp);
        
        chunks
    }
    
    /// Fill empty cell with value and update possible for surrounding cells
    fn fill_blank_cell(&mut self, cell_index: usize, cell_val: u32) {
        self.board[cell_index] = char::from_digit(cell_val, 10).unwrap();

        let chunks_guess = self.get_chunks_containing_cell(cell_index);
        for chunk in chunks_guess {
            for &i in chunk.iter() {
                self.possible[i] &= !(!0 << 9) & !(1 << (cell_val-1));
            }
        }
    }

    /// Go through each chunk and find missing nums, then add them to poss using or
    fn update_possible(&mut self) {
        //let mut chunk_val_index: [(char, usize); 9] = Default::default();
        let mut chunk: [char; 9] = Default::default();
        let mut chunk_indices: [usize; 9] = Default::default();
        let mut chunk_possible: u32;
        let mut cell_index : usize;
        let mut chunk_index;

        // compute rows
        for row_start in (0..81).step_by(9) {
            chunk.copy_from_slice(&self.board[row_start..row_start+9]);
            chunk_possible = self.get_possible_from_chunk(chunk);

            for row_offset in 0..9 {
                self.possible[row_start+row_offset] = chunk_possible;
            }
        }

        // compute columns
        for col_start in 0..9 {
            chunk_index = 0;
            for col_offset in (0..81).step_by(9) {
                cell_index = col_start+col_offset;
                chunk[col_offset/9] = self.board[cell_index];
                
                chunk_indices[chunk_index] = cell_index;
                chunk_index += 1;
            }

            chunk_possible = self.get_possible_from_chunk(chunk);
            for &cell_index in chunk_indices.iter() {
                self.possible[cell_index] &= chunk_possible;
            }
        }
        

        // compute squares
        for grid_start in [0, 3, 6, 27, 30, 33, 54, 57, 60].iter(){
            chunk_index = 0;
            for grid_offset1 in (0..19).step_by(9) {
                for grid_offset2 in 0..3 {
                    cell_index = grid_start + grid_offset1 + grid_offset2;
                    chunk[chunk_index] = self.board[cell_index];
                    chunk_indices[chunk_index] = cell_index;
                    chunk_index += 1;
                }
            }
            
            chunk_possible = self.get_possible_from_chunk(chunk);
            
            for &cell_index in chunk_indices.iter() {
                self.possible[cell_index] &= chunk_possible;
            }
        }
    }
    
    /// Convert from possible u32 to vec containing possible values
    fn possible_to_vec(&self, mut possible : u32) -> Vec<u32> {
        let mut poss_vec: Vec<u32> = Vec::new();
        let mut x;
        
        while possible > 0 {
            x = possible.trailing_zeros()+1;
            poss_vec.push(x);
            possible -= 1 << (x-1);
        }
        
        poss_vec
    }
    
    fn solve_naked_singles(&mut self, blank_indicies: &mut Vec<usize>) {
        blank_indicies.retain(|&cell_num| {
            if self.possible[cell_num].count_ones() == 1 {
                let cell_val = self.possible_to_vec(self.possible[cell_num])[0];
                
                self.fill_blank_cell(cell_num, cell_val);

                false
            } else {
                true
            }
        });
    }

    fn solve_naked_pairs(&mut self) {
        let chunks = self.get_chunks();

        for chunk in chunks {
            for i in 0..chunk.len() {
                if self.board[chunk[i]] == '.' && self.possible[chunk[i]].count_ones() == 2 {
                    for j in i+1..chunk.len() {
                        if self.possible[chunk[i]] == self.possible[chunk[j]] && self.board[chunk[j]] == '.' {
                            for k in 0..chunk.len() {
                                if k == i || k == j { continue; }
                                self.possible[chunk[k]] &= !(!0 << 9) & !self.possible[chunk[i]];
                            }
                        }
                    }
                }
            }
        }
    }

    fn solve_hidden_singles(&mut self, blank_indicies: &mut Vec<usize>) {
        blank_indicies.retain(|&cell_num| {
            let poss_vec = self.possible_to_vec(self.possible[cell_num]);
            let chunks_nearby = self.get_chunks_containing_cell(cell_num);

            for poss in poss_vec {
                for chunk in &chunks_nearby {
                    let mut found_other = false;

                    for &index in chunk {
                        let contains_poss = self.possible[index] & (1 << (poss-1)) > 0;
                        if index != cell_num && self.board[index] == '.' && contains_poss {
                            found_other = true;
                            break;
                        }
                    }

                    if found_other { break; }
                    
                    self.fill_blank_cell(cell_num, poss);

                    return false;
                }
            }
            true
        });
    }

    /// Check if the board is valid and does not contain chunks with the same cell
    pub fn is_valid(&self) -> bool {
        let chunks = self.get_chunks();

        let mut check_values: Vec<u32>;

        for chunk in &chunks {
            check_values = (1..10).collect();

            for i in 0..chunk.len() {
                if self.board[i] != '.' {
                    let value = self.board[i].to_digit(10).unwrap();
                    
                    if check_values.contains(&value) {
                        check_values.retain(|&x| x != value);
                    } else {
                        return false;
                    }
                }
            }
        }

        true
    }

    /// Check if the board is solved and does not contain any blank cells
    pub fn is_solved(&self) -> bool {
        if self.board.contains(&'.') { false }
        else {
            let chunks = self.get_chunks();

            for chunk in &chunks {
                let mut check_values: Vec<u32> = (1..10).collect();

                for i in 0..chunk.len() {
                    if self.board[i] == '.' { return false; }

                    for j in 0..check_values.len() {
                        if check_values[j] == self.board[i].to_digit(10).unwrap() {
                            check_values.remove(j);
                            break;
                        }
                    }
                }

                if check_values.len() != 0 { return false; }
            }

            true
        }
    } 
}

impl fmt::Display for Sudoku {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {

        for row in self.board.chunks(9) {
            for &cell in row {
                write!(f, "{}", cell)?;
            }
            write!(f, "\n")?;
        }

        Ok(())   
    }
}