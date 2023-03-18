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

struct Column {
    name: String,
    width: i32,
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
        label("View: All", 3, 4, WHITE_PAIR);
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

    fn draw_curr(&self) {
        let start_y: usize = 4;
        for (col_num, item) in self.data[self.curr].iter().enumerate() {
            label(
                &format!("[{}|{}]", col_num, self.schema[col_num].name),
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
        }
    }

    fn up(&mut self, by: i32) {
        if self.curr as i32 - by >= 0 {
            self.curr -= by as usize;
        } else {
            self.curr = 0;
        }
    }

    fn down(&mut self, by: usize) {
        if self.curr + by < self.data.len() {
            self.curr += by;
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
            TableFocus::Element => table.draw_curr(),
            _ => todo!(),
        };

        let key = getch();
        match key as u8 as char {
            'q' => quit = true,
            'w' => todo!(),
            'j' => match table_focus {
                TableFocus::Table => table.down(1),
                _ => {}
            },
            'k' => match table_focus {
                TableFocus::Table => table.up(1),
                _ => {}
            },
            'J' => match table_focus {
                TableFocus::Table => table.down(6),
                _ => {}
            },
            'K' => match table_focus {
                TableFocus::Table => table.up(6),
                _ => {}
            },
            'h' => {}
            'l' => {}
            'v' => match table_focus {
                TableFocus::Table => table_focus = TableFocus::View,
                _ => {}
            },
            'c' => match table_focus {
                TableFocus::Table => table_focus = TableFocus::Column,
                _ => {}
            },
            's' => match table_focus {
                TableFocus::Table => table_focus = TableFocus::Sort,
                _ => {}
            },
            'V' => {}
            'n' => table.switch_num_mode(),
            'i' => {}
            'f' => {}
            'u' => {}
            '\n' => match table_focus {
                TableFocus::Table => table_focus = TableFocus::Element,
                _ => table_focus = TableFocus::Table,
            },
            ':' => {}
            '=' => {}
            '?' => {}
            _ => {}
        }
    }

    endwin();
}
