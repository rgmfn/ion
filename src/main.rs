use ncurses::*;
use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};
// use serde_json::Result;
use std::fs::File;
use std::io::Write;
use std::{cmp::max, fs};

const WHITE_PAIR: i16 = 0;
const INV_WHITE_PAIR: i16 = 1;
const RED_PAIR: i16 = 2;
const INV_RED_PAIR: i16 = 3;
const GREEN_PAIR: i16 = 4;
const INV_GREEN_PAIR: i16 = 5;
const YELLOW_PAIR: i16 = 6;
const INV_YELLOW_PAIR: i16 = 7;
const BLUE_PAIR: i16 = 8;
const INV_BLUE_PAIR: i16 = 9;
const MAGENTA_PAIR: i16 = 10;
const INV_MAGENTA_PAIR: i16 = 11;
const CYAN_PAIR: i16 = 12;
const INV_CYAN_PAIR: i16 = 13;

#[derive(Clone, Copy)]
enum InputMode {
    Normal,
    Text,
    // Cmd, ?
}

#[derive(Serialize, Deserialize)]
struct Column {
    name: String,
    width: i32,
    // type: X,
    // default: value,
}

fn label(text: &str, y: i32, x: i32, pair: i16) {
    mv(y, x);
    attron(COLOR_PAIR(pair));
    addstr(text);
    attroff(COLOR_PAIR(pair));
}

#[derive(Serialize, Deserialize)]
enum TableFocus {
    Table,
    Element,
    NewElement,
    View,
    Sort,
    Column,
    NewColumn,
}

#[derive(Serialize, Deserialize)]
struct Table {
    title: String,
    subtitle: String,
    // views: Vec<View>,
    columns: Vec<Column>,
    data: Vec<Vec<String>>,
    curr_elem: usize,
    curr_col: usize,
    num_mode: NumMode,
    table_focus: TableFocus,
}

impl Table {
    fn draw_title(&self) {
        label(&self.title, 0, 0, WHITE_PAIR);
    }

    fn draw_subtitle(&self) {
        label(&self.subtitle, 2, 4, WHITE_PAIR);
    }

    fn draw_views(&self) {
        label("View: ", 3, 4, WHITE_PAIR);
        attron(COLOR_PAIR(INV_WHITE_PAIR));
        addstr("All");
        attroff(COLOR_PAIR(INV_WHITE_PAIR));
    }

    fn draw_headers(&self) {
        let num_col_size: usize = (self.data.len() as f32).log10() as usize + 1;
        {
            label("+", 4, 4, WHITE_PAIR);
            addstr(&n_of_c(num_col_size + 2, '-'));
            addstr("+");

            for col in self.columns.iter() {
                addstr(&format!("{}+", n_of_c((col.width + 2) as usize, '-')));
            }
        }
        {
            label(
                &format!("| {} ", n_of_c(num_col_size, ' ')),
                5,
                4,
                WHITE_PAIR,
            );

            for (col_num, col) in self.columns.iter().enumerate() {
                let pair = match self.table_focus {
                    TableFocus::Column => {
                        if col_num == self.curr_col {
                            INV_WHITE_PAIR
                        } else {
                            WHITE_PAIR
                        }
                    }
                    _ => WHITE_PAIR,
                };
                addstr("| ");
                attron(COLOR_PAIR(pair));
                addstr(&format!("{}", col.name));
                attroff(COLOR_PAIR(pair));
                addstr(&n_of_c(col.width as usize - col.name.len(), ' '));
                addstr(" ");
            }
            addstr("|");
        }
        {
            // TODO is duplicate of 1st block
            label("+", 6, 4, WHITE_PAIR);
            addstr(&n_of_c(num_col_size + 2, '='));
            addstr("+");

            for col in self.columns.iter() {
                addstr(&format!("{}+", n_of_c((col.width + 2) as usize, '=')));
            }
        }
    }

    fn draw_data(&self) {
        let start_y: i32 = 7;
        let num_col_size: usize = (self.data.len() as f32).log10() as usize + 1;
        for (row_num, row) in self.data.iter().enumerate() {
            // freak out if row longer than columns

            let pair: i16 = if row_num == self.curr_elem as usize {
                INV_WHITE_PAIR
            } else {
                WHITE_PAIR
            };
            mv(row_num as i32 + start_y, 4);
            attron(COLOR_PAIR(pair));
            {
                addstr("| ");
                addstr(&fit_to_sizer(
                    &format!(
                        "{} ",
                        match self.num_mode {
                            NumMode::Absolute => row_num + 1,
                            NumMode::Relative =>
                                (row_num as i32 - self.curr_elem as i32).abs() as usize,
                        }
                    ),
                    num_col_size + 1,
                    ' ',
                ));
            }
            for (col_num, item) in row.iter().enumerate() {
                addstr(&fit_to_sizel(
                    &format!("| {} ", item),
                    self.columns[col_num].width as usize + 3,
                    ' ',
                ));
            }
            addstr("|");
            attroff(COLOR_PAIR(pair));
        }
    }

    fn draw_footer(&self) {
        {
            label("+", 7 + self.data.len() as i32, 4, WHITE_PAIR);
            let num_col_size: usize = (self.data.len() as f32).log10() as usize + 1;
            addstr(&n_of_c(num_col_size + 2, '-'));
            addstr("+");

            for col in self.columns.iter() {
                addstr(&format!("{}+", n_of_c((col.width + 2) as usize, '-')));
            }
            label(
                &format!(
                    "{} {}",
                    self.data.len(),
                    if self.data.len() == 1 {
                        "entry"
                    } else {
                        "entries"
                    }
                ),
                8 + self.data.len() as i32,
                5,
                WHITE_PAIR,
            );
        }
    }

    fn draw_elem(&self, motion_num: usize, input_mode: InputMode, input_str: &str) {
        let start_y: usize = 4;
        for (col_num, item) in self.data[self.curr_elem].iter().enumerate() {
            label(
                &format!("[{}|{}]", col_num + 1, self.columns[col_num].name),
                (start_y + col_num * 3) as i32,
                4,
                WHITE_PAIR,
            );
            label(
                &format!("{}", item),
                (start_y + col_num * 3 + 1) as i32,
                6,
                WHITE_PAIR,
            );
            match input_mode {
                InputMode::Text => {
                    if motion_num == col_num + 1 {
                        addstr(&format!(" -> {}", input_str));
                    }
                }
                _ => {}
            }
        }
    }

    fn draw_column(&self, motion_num: usize, input_mode: InputMode, input_str: &str) {
        let start_y: usize = 8;
        let col = &self.columns[self.curr_col];
        label("[1|name]: ", start_y as i32, 8, WHITE_PAIR);
        addstr(&format!("{}", col.name));
        match input_mode {
            InputMode::Text if motion_num == 1 => _ = addstr(&format!(" -> {}", input_str)),
            _ => {}
        };
        label("[2|width]:  ", start_y as i32 + 1, 8, WHITE_PAIR);
        addstr(&format!("{}", col.width));
        match input_mode {
            InputMode::Text if motion_num == 2 => _ = addstr(&format!(" -> {}", input_str)),
            _ => {}
        };
        label("[3|type]:  String", start_y as i32 + 2, 8, WHITE_PAIR);
        // addstr(&format!("{}", col.name));
        // match input_mode {
        //     InputMode::Text if motion_num == 2 => _ = addstr(&format!(" -> {}", input_str)),
        //     _ => {}
        // };
    }

    fn to_new_elem(&mut self) {
        self.data.push(vec!["".to_string(); self.columns.len()]);
        self.table_focus = TableFocus::NewElement;
    }

    fn to_view(&mut self) {
        self.table_focus = TableFocus::View;
    }

    fn to_sort(&mut self) {
        self.table_focus = TableFocus::Sort;
    }

    fn to_table(&mut self) {
        self.table_focus = TableFocus::Table;
    }

    fn to_col(&mut self) {
        self.table_focus = TableFocus::Column;
        self.curr_col = 0;
    }

    fn to_new_col(&mut self) {
        let new_col = Column {
            name: "".to_string(),
            width: 0,
        };
        self.columns.push(new_col);

        for row_num in 0..self.data.len() {
            self.data[row_num].push("".to_string());
        }

        // push new column
        // add to each row in data
        self.table_focus = TableFocus::NewColumn;
    }

    fn to_curr_elem(&mut self) {
        self.table_focus = TableFocus::Element;
    }

    fn up(&mut self, by: i32, def: i32) {
        let n: i32 = if by == 0 { def } else { by };
        if self.curr_elem as i32 - n >= 0 {
            self.curr_elem -= n as usize;
        } else {
            self.curr_elem = 0;
        }
    }

    fn down(&mut self, by: usize, def: usize) {
        let n: usize = if by == 0 { def } else { by };
        if self.curr_elem + n < self.data.len() {
            self.curr_elem += n;
        } else {
            self.curr_elem = self.data.len() - 1;
        }
    }

    fn set_curr_elem(&mut self, to: i32) {
        if to > 0 && to <= self.data.len() as i32 {
            self.curr_elem = to as usize - 1;
        }
    }

    fn switch_num_mode(&mut self) {
        match self.num_mode {
            NumMode::Absolute => self.num_mode = NumMode::Relative,
            NumMode::Relative => self.num_mode = NumMode::Absolute,
        }
    }

    fn prev_col(&mut self) {
        if self.curr_col > 0 {
            self.curr_col -= 1;
        } else {
            self.curr_col = self.columns.len() - 1;
        }
    }

    fn next_col(&mut self) {
        if self.curr_col + 1 < self.columns.len() {
            self.curr_col += 1;
        } else {
            self.curr_col = 0;
        }
    }

    fn grow_curr_col(&mut self, motion_num: usize) {
        let amount: i32 = if motion_num > 0 { motion_num as i32 } else { 1 };
        self.columns[self.curr_col].width += amount;
    }

    fn shrink_curr_col(&mut self, motion_num: i32) {
        let amount: i32 = if motion_num > 0 { motion_num } else { 1 };
        if self.columns[self.curr_col].width - amount
            > self.columns[self.curr_col].name.len() as i32
        {
            self.columns[self.curr_col].width -= amount;
        } else {
            self.columns[self.curr_col].width = amount;
        }
    }

    fn auto_size_col(&mut self, col: usize) {
        let mut min_size = self.columns[col].name.len();
        for row in self.data.iter() {
            min_size = max(min_size, row[col].len());
        }
        self.columns[col].width = min_size as i32;
    }

    fn auto_size_curr_col(&mut self) {
        self.auto_size_col(self.curr_col);
    }

    fn auto_size_cols(&mut self) {
        let num_cols = self.columns.len();
        for col_num in 0..num_cols {
            self.auto_size_col(col_num);
        }
    }

    fn del_curr_elem(&mut self) {
        _ = self.data.remove(self.curr_elem);
        if self.curr_elem + 1 > self.data.len() {
            self.curr_elem = self.data.len() - 1;
        }
        self.table_focus = TableFocus::Table;
    }
}

fn n_of_c(n: usize, c: char) -> String {
    std::iter::repeat(c).take(n).collect::<String>()
}

fn fit_to_sizel(text: &str, n: usize, pad: char) -> String {
    if n >= text.len() {
        let mut ret = "".to_string();
        ret.push_str(text);
        ret.push_str(&n_of_c(n - text.len(), pad));

        ret
    } else {
        format!("{}..", &text[..(n - 2)])
    }
}

fn fit_to_sizer(text: &str, n: usize, pad: char) -> String {
    if n > text.len() {
        let mut ret = "".to_string();
        ret.push_str(&n_of_c(n - text.len(), pad));
        ret.push_str(text);

        ret
    } else {
        text.to_string()
    }
}

#[derive(Serialize, Deserialize)]
enum NumMode {
    Absolute,
    Relative,
}

// fn write_table(table: &Table) -> Result<()> {
//     let j = serde_json::to_string(&table)?;
//     println!("{}", j);
//     Ok(())
// }

// TODO delete a column
// TODO give columns default values
// TODO e to edit values?
// TODO add/save data
// TODO create different types for column values, not just strings
// TODO undo system (hosted in hidden file? so it persists)

fn untyped_example() -> Result<()> {
    // let data = r#"
    //     {
    //         "name": "John Doe",
    //         "age": 43,
    //         "phones": [
    //             "+44 1235467",
    //             "+44 2345678"
    //         ]
    //     }"#;
    // let data = "{\n\"name\": \"John Doe\",\n\"age\": 43,\n\"phones\": [\n\"+44 1234567\",\n\"+44 2345678\"\n]\n}";
    let data: String = fs::read_to_string(".classes.json").expect("unable to read file");

    println!("try");
    let res: Result<Value> = serde_json::from_str(&data);

    let val: Value = match res {
        Ok(v) => v,
        Err(error) => panic!("Problem reading json: {:?}", error),
    };

    println!(
        "Please call {} at number {}",
        val["title"], val["data"][1][2]
    );
    Ok(())
}

fn load_table(file_str: &str) -> Table {
    let table_str: String = fs::read_to_string(file_str).expect("No file to read");
    let res: Result<Table> = serde_json::from_str(&table_str);
    let table: Table = match res {
        Ok(t) => t,
        Err(error) => panic!("Problem reading json: {:?}", error),
    };

    table
}

fn save_table(table: Table, file_str: &str) -> Result<()> {
    let mut file = File::create(file_str).unwrap();
    let j = serde_json::to_string(&table)?;
    writeln!(file, "{}", j);
    Ok(())
}

fn main() {
    initscr();
    noecho();
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);

    start_color();
    init_pair(WHITE_PAIR, COLOR_WHITE, COLOR_BLACK);
    init_pair(INV_WHITE_PAIR, COLOR_BLACK, COLOR_WHITE);
    init_pair(RED_PAIR, COLOR_RED, COLOR_BLACK);
    init_pair(INV_RED_PAIR, COLOR_BLACK, COLOR_RED);
    init_pair(GREEN_PAIR, COLOR_GREEN, COLOR_BLACK);
    init_pair(INV_GREEN_PAIR, COLOR_BLACK, COLOR_GREEN);
    init_pair(YELLOW_PAIR, COLOR_YELLOW, COLOR_BLACK);
    init_pair(INV_YELLOW_PAIR, COLOR_BLACK, COLOR_YELLOW);
    init_pair(BLUE_PAIR, COLOR_BLUE, COLOR_BLACK);
    init_pair(INV_BLUE_PAIR, COLOR_BLACK, COLOR_BLUE);
    init_pair(MAGENTA_PAIR, COLOR_MAGENTA, COLOR_BLACK);
    init_pair(INV_MAGENTA_PAIR, COLOR_BLACK, COLOR_MAGENTA);
    init_pair(CYAN_PAIR, COLOR_CYAN, COLOR_BLACK);
    init_pair(INV_CYAN_PAIR, COLOR_BLACK, COLOR_CYAN);

    let mut table: Table = load_table("table.json");
    let mut input_mode: InputMode = InputMode::Normal;
    let mut input_str: String = "".to_string();
    let mut motion_num: usize = 0;

    let mut screen_w = 0;
    let mut screen_h = 0;
    getmaxyx(stdscr(), &mut screen_h, &mut screen_w);

    let mut preserve_motion: bool = false;

    let mut quit = false;
    while !quit {
        erase();

        table.draw_title();
        table.draw_subtitle();

        match table.table_focus {
            TableFocus::Table => {
                table.draw_data();
                table.draw_views();
                table.draw_headers();
                table.draw_footer();
            }
            TableFocus::Element => table.draw_elem(motion_num as usize, input_mode, &input_str),
            TableFocus::NewElement => table.draw_elem(motion_num, InputMode::Text, &input_str),
            TableFocus::Column => {
                table.draw_headers();
                table.draw_column(motion_num as usize, input_mode, &input_str)
            }
            TableFocus::NewColumn => {
                table.draw_headers();
                table.draw_column(motion_num as usize, InputMode::Text, &input_str)
            }
            _ => todo!(),
        };

        if motion_num != 0 {
            mv(screen_h - 1, 0);
            addstr(&format!("{}", motion_num));
        }

        let key = getch();
        match input_mode {
            InputMode::Normal => match table.table_focus {
                TableFocus::Table => match key as u8 as char {
                    'q' | '\x1b' => quit = true,
                    'j' => table.down(motion_num, 1),
                    'k' => table.up(motion_num as i32, 1),
                    'J' => table.down(motion_num, 10),
                    'K' => table.up(motion_num as i32, 10),
                    'G' => table.set_curr_elem(motion_num as i32),
                    'c' => table.to_col(),
                    's' => table.to_sort(),
                    'v' => table.to_view(),
                    // 'V' => {}
                    'n' => table.switch_num_mode(),
                    'i' => {
                        table.to_new_elem();
                        motion_num = 1;
                        table.curr_elem = table.data.len() - 1;
                        input_mode = InputMode::Text;
                        input_str = "".to_string();
                        preserve_motion = true;
                    }
                    'd' => table.del_curr_elem(),
                    '\n' => table.to_curr_elem(),
                    '=' => table.auto_size_cols(),
                    '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' | '0' => {
                        motion_num = motion_num * 10 + (key as usize - 48);
                        preserve_motion = true;
                    }
                    '\x08' | '\x7f' => {
                        motion_num /= 10;
                        preserve_motion = true;
                    }
                    _ => {}
                },
                TableFocus::Element => match key as u8 as char {
                    'q' | '\x1b' => table.to_table(),
                    'j' => table.down(motion_num, 1),
                    'k' => table.up(motion_num as i32, 1),
                    'd' => table.del_curr_elem(),
                    '\n' if motion_num > 0 => {
                        input_mode = InputMode::Text;
                        input_str = "".to_string();
                        preserve_motion = true;
                    }
                    '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' | '0' => {
                        motion_num = motion_num * 10 + (key as usize - 48);
                        preserve_motion = true;
                    }
                    '\x08' | '\x7f' => {
                        motion_num /= 10;
                        preserve_motion = true;
                    }
                    _ => {}
                },
                TableFocus::Column => match key as u8 as char {
                    'q' | '\x1b' => table.to_table(),
                    'h' => table.prev_col(),
                    'l' => table.next_col(),
                    'c' => table.to_table(),
                    '=' => table.auto_size_curr_col(),
                    '+' => table.grow_curr_col(motion_num),
                    '-' => table.shrink_curr_col(motion_num as i32),
                    'i' => {
                        table.to_new_col();
                        motion_num = 1;
                        table.curr_col = table.columns.len() - 1;
                        input_mode = InputMode::Text;
                        input_str = "".to_string();
                        preserve_motion = true;
                    }
                    '\n' if motion_num > 0 => {
                        input_mode = InputMode::Text;
                        input_str = "".to_string();
                        preserve_motion = true;
                    }
                    '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' | '0' => {
                        motion_num = motion_num * 10 + (key as usize - 48);
                        preserve_motion = true;
                    }
                    '\x08' | '\x7f' => {
                        motion_num /= 10;
                        preserve_motion = true
                    }
                    _ => {}
                },
                _ => {}
            },
            InputMode::Text => match key as u8 as char {
                '\n' => match table.table_focus {
                    TableFocus::Element => {
                        table.data[table.curr_elem].push(input_str);
                        table.data[table.curr_elem].swap_remove(motion_num - 1);
                        input_str = "".to_string();
                        input_mode = InputMode::Normal;
                    }
                    TableFocus::NewElement => {
                        let table_len = table.data.len();
                        table.data[table_len - 1].push(input_str);
                        table.data[table_len - 1].swap_remove(motion_num - 1);
                        input_str = "".to_string();

                        if motion_num < table.columns.len() {
                            motion_num += 1;
                            preserve_motion = true;
                        } else {
                            table.curr_elem = table_len - 1;
                            table.to_table();
                            input_mode = InputMode::Normal;
                        }
                    }
                    TableFocus::Column => match motion_num {
                        1 => {
                            let new_str_len = input_str.len() as i32;
                            table.columns[table.curr_col].name = input_str;
                            table.columns[table.curr_col].width =
                                max(table.columns[table.curr_col].width, new_str_len);
                            input_str = "".to_string();
                            input_mode = InputMode::Normal;
                        }
                        2 => {
                            // TODO input validation
                            table.columns[table.curr_col].width = max(
                                input_str.parse::<i32>().unwrap(),
                                table.columns[table.curr_col].name.len() as i32,
                            );
                            input_str = "".to_string();
                            input_mode = InputMode::Normal;
                        }
                        // 3 => {}
                        _ => {}
                    },
                    TableFocus::NewColumn => {
                        match motion_num {
                            1 => {
                                if !input_str.is_empty() {
                                    let input_len: i32 = input_str.len() as i32;
                                    table.columns[table.curr_col].name = input_str;
                                    table.columns[table.curr_col].width = input_len;
                                    input_str = "".to_string();
                                    motion_num += 1;
                                }
                                preserve_motion = true;
                            }
                            2 => {
                                if !input_str.is_empty() {
                                    table.columns[table.curr_col].width = max(
                                        input_str.parse::<i32>().unwrap(),
                                        table.columns[table.curr_col].name.len() as i32,
                                    );
                                }
                                input_str = "".to_string();
                                table.to_table();
                                input_mode = InputMode::Normal;
                            }
                            // 3 => {
                            //     motion_num += 1;
                            // }
                            _ => todo!(),
                        };
                    }
                    _ => {}
                },
                '\x1b' => {
                    input_mode = InputMode::Normal;
                    table.to_table();
                }
                '\x08' | '\x7f' => {
                    input_str.pop();
                    preserve_motion = true;
                } // backspace
                _ => {
                    input_str.push_str(&(key as u8 as char).to_string());
                    preserve_motion = true;
                }
            },
        }

        if !preserve_motion {
            motion_num = 0;
        } else {
            preserve_motion = false;
        }
    }
    save_table(table, "table.json");

    endwin();
}
