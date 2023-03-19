use ncurses::*;

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
    // CMD,
}

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

enum TableFocus {
    Table,
    Element,
    NewElement,
    View,
    Sort,
    Column,
}

struct Table {
    title: String,
    subtitle: String,
    // views: Vec<View>,
    schema: Vec<Column>,
    data: Vec<Vec<String>>,
    curr: usize,
    num_mode: NumMode,
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

            for col in self.schema.iter() {
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

            for col in self.schema.iter() {
                addstr("|");
                addstr(&format!(
                    " {} ",
                    fit_to_sizel(&col.name, col.width as usize, ' ')
                ));
            }
            addstr("|");
        }
        {
            // TODO is duplicate of 1st block
            label("+", 6, 4, WHITE_PAIR);
            addstr(&n_of_c(num_col_size + 2, '='));
            addstr("+");

            for col in self.schema.iter() {
                addstr(&format!("{}+", n_of_c((col.width + 2) as usize, '=')));
            }
        }
    }

    fn draw_data(&self) {
        let start_y: i32 = 7;
        let num_col_size: usize = (self.data.len() as f32).log10() as usize + 1;
        for (row_num, row) in self.data.iter().enumerate() {
            // freak out if row longer than schema

            let pair: i16 = if row_num == self.curr as usize {
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
                            NumMode::ABSOLUTE => row_num,
                            NumMode::RELATIVE => (row_num as i32 - self.curr as i32).abs() as usize,
                        }
                    ),
                    num_col_size + 1,
                    ' ',
                ));
            }
            for (col_num, item) in row.iter().enumerate() {
                addstr(&fit_to_sizel(
                    &format!("| {} ", item),
                    self.schema[col_num].width as usize + 3,
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

            for col in self.schema.iter() {
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

    fn draw_elem(&self, row_num: usize, motion_num: usize, input_mode: InputMode, input_str: &str) {
        let start_y: usize = 4;
        for (col_num, item) in self.data[row_num].iter().enumerate() {
            label(
                &format!("[{}|{}]", col_num + 1, self.schema[col_num].name),
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

    fn up(&mut self, by: i32, def: i32) {
        let n: i32 = if by == 0 { def } else { by };
        if self.curr as i32 - n >= 0 {
            self.curr -= n as usize;
        } else {
            self.curr = 0;
        }
    }

    fn down(&mut self, by: usize, def: usize) {
        let n: usize = if by == 0 { def } else { by };
        if self.curr + n < self.data.len() {
            self.curr += n;
        } else {
            self.curr = self.data.len() - 1;
        }
    }

    fn switch_num_mode(&mut self) {
        match self.num_mode {
            NumMode::ABSOLUTE => self.num_mode = NumMode::RELATIVE,
            NumMode::RELATIVE => self.num_mode = NumMode::ABSOLUTE,
        }
    }
}

fn n_of_c(n: usize, c: char) -> String {
    std::iter::repeat(c).take(n).collect::<String>()
}

fn fit_to_sizel(text: &str, n: usize, pad: char) -> String {
    if n > text.len() {
        let mut ret = "".to_string();
        ret.push_str(text);
        ret.push_str(&n_of_c(n - text.len(), pad));

        ret
    } else {
        text.to_string()
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

enum NumMode {
    ABSOLUTE,
    RELATIVE,
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

    let schema: Vec<Column> = vec![
        Column {
            name: "Course".to_string(),
            width: 20,
        },
        Column {
            name: "Name".to_string(),
            width: 20,
        },
        Column {
            name: "Date".to_string(),
            width: 20,
        },
        Column {
            name: "Status".to_string(),
            width: 20,
        },
        Column {
            name: "Type".to_string(),
            width: 20,
        },
    ];
    let data: Vec<Vec<String>> = vec![
        vec![
            "CSE 115A".to_string(),
            "Essay".to_string(),
            "03/17/23".to_string(),
            "Not Started".to_string(),
            "Paper".to_string(),
        ],
        vec![
            "CSE 180".to_string(),
            "Final".to_string(),
            "03/23/23".to_string(),
            "Not Started".to_string(),
            "Exam".to_string(),
        ],
        vec![
            "CSE 115A".to_string(),
            "TSR 4".to_string(),
            "02/14/23".to_string(),
            "Completed".to_string(),
            "Assignment".to_string(),
        ],
        vec![
            "CSE 180".to_string(),
            "Gradiance 1".to_string(),
            "01/26/23".to_string(),
            "Completed".to_string(),
            "Assingment".to_string(),
        ],
    ];

    let mut table: Table = Table {
        title: "Spring 2021 Schedule".to_string(),
        subtitle: "Spring 2021 Schedule Subtitle".to_string(),
        schema,
        data,
        curr: 0,
        num_mode: NumMode::ABSOLUTE,
    };
    let mut table_focus: TableFocus = TableFocus::Table;

    let mut input_mode: InputMode = InputMode::Normal;
    let mut input_str: String = "".to_string();
    let mut motion_num: usize = 0;

    let mut screen_w = 0;
    let mut screen_h = 0;
    getmaxyx(stdscr(), &mut screen_h, &mut screen_w);

    let mut quit = false;
    while !quit {
        erase();

        table.draw_title();
        table.draw_subtitle();

        match table_focus {
            TableFocus::Table => {
                table.draw_data();
                table.draw_views();
                table.draw_headers();
                table.draw_footer();
            }
            TableFocus::Element => {
                table.draw_elem(table.curr, motion_num as usize, input_mode, &input_str)
            }
            TableFocus::NewElement => table.draw_elem(
                table.data.len() - 1,
                motion_num,
                InputMode::Text,
                &input_str,
            ),
            _ => todo!(),
        };

        if motion_num != 0 {
            mv(screen_h - 1, 0);
            addstr(&format!("{}", motion_num));
            // label(&format!("{}", motion_num), screen_h, 4, WHITE_PAIR);
        }

        let key = getch();
        match input_mode {
            InputMode::Text => match key as u8 as char {
                '\n' | '\r' => match table_focus {
                    TableFocus::Element => {
                        table.data[table.curr].push(input_str);
                        table.data[table.curr].swap_remove(motion_num - 1);
                        input_str = "".to_string();
                        motion_num = 0;
                        input_mode = InputMode::Normal;
                    }
                    TableFocus::NewElement => {
                        if motion_num < table.schema.len() {
                            let table_len = table.data.len();
                            table.data[table_len - 1].push(input_str);
                            table.data[table_len - 1].swap_remove(motion_num - 1);
                            input_str = "".to_string();
                            motion_num += 1;
                        } else {
                            let table_len = table.data.len();
                            table.data[table_len - 1].push(input_str);
                            table.data[table_len - 1].swap_remove(motion_num - 1);
                            table.curr = table_len - 1;
                            table_focus = TableFocus::Table;
                            input_mode = InputMode::Normal;
                            motion_num = 0;
                            input_str = "".to_string();
                        }
                    }
                    _ => {}
                }, // enter
                '\x1b' => quit = true,                  // escape
                '\x08' | '\x7f' => _ = input_str.pop(), // backspace
                _ => input_str.push_str(&(key as u8 as char).to_string()),
            },
            InputMode::Normal => match key as u8 as char {
                'q' => quit = true,
                'w' => todo!(),
                'j' => match table_focus {
                    TableFocus::Table => {
                        table.down(motion_num, 1);
                        motion_num = 0;
                    }
                    _ => {}
                },
                'k' => match table_focus {
                    TableFocus::Table => {
                        table.up(motion_num as i32, 1);
                        motion_num = 0;
                    }
                    _ => {}
                },
                'J' => match table_focus {
                    TableFocus::Table => {
                        table.down(motion_num, 10);
                        motion_num = 0;
                    }
                    _ => {}
                },
                'K' => match table_focus {
                    TableFocus::Table => {
                        table.up(motion_num as i32, 10);
                        motion_num = 0;
                    }
                    _ => {}
                },
                'h' => {}
                'l' => {}
                'v' => match table_focus {
                    TableFocus::Table => {
                        table_focus = TableFocus::View;
                        motion_num = 0;
                    }
                    _ => {}
                },
                'c' => match table_focus {
                    TableFocus::Table => {
                        table_focus = TableFocus::Column;
                        motion_num = 0;
                    }
                    _ => {}
                },
                's' => match table_focus {
                    TableFocus::Table => {
                        table_focus = TableFocus::Sort;
                        motion_num = 0;
                    }
                    _ => {}
                },
                'V' => {}
                'n' => table.switch_num_mode(),
                'i' => match table_focus {
                    TableFocus::Table => {
                        table.data.push(vec!["".to_string(); table.schema.len()]);
                        table_focus = TableFocus::NewElement;
                        motion_num = 1;
                        input_mode = InputMode::Text;
                        // todo!(); // insert new row into table.data
                    }
                    _ => {}
                },
                'f' => {}
                'u' => {}
                '\n' => match table_focus {
                    TableFocus::Table => {
                        table_focus = TableFocus::Element;
                        motion_num = 0;
                    }
                    TableFocus::Element => {
                        if motion_num > 0 {
                            input_mode = InputMode::Text;
                        }
                    }
                    TableFocus::NewElement => {
                        todo!() // place in new string
                    }
                    _ => {}
                },
                '\x1b' => table_focus = TableFocus::Table,
                ':' => {}
                '=' => {}
                '?' => {}
                '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' | '0' => {
                    motion_num = motion_num * 10 + (key as usize - 48);
                }
                _ => {
                    println!("{}", key)
                }
            },
        }
    }

    endwin();
}
