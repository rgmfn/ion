use ncurses::*;

const TEXT_PAIR: i16 = 0;
const INV_TEXT_PAIR: i16 = 1;

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

struct Table {
    title: String,
    subtitle: String,
    // views: Vec<View>,
    schema: Vec<Column>,
    data: Vec<Vec<String>>,
    curr: i32,
    num_mode: NumMode,
}

impl Table {
    fn draw_title(&self) {
        label(&self.title, 0, 0, TEXT_PAIR);
    }

    fn draw_subtitle(&self) {
        label(&self.subtitle, 2, 4, TEXT_PAIR);
    }

    fn draw_views(&self) {
        label("View: All", 3, 4, TEXT_PAIR);
    }

    fn draw_headers(&self) {
        let num_col_size: usize = (self.data.len() as f32).log10() as usize + 1;
        {
            label("+", 4, 4, TEXT_PAIR);
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
                TEXT_PAIR,
            );

            for col in self.schema.iter() {
                addstr(&format!(
                    "| {} ",
                    fit_to_sizel(&col.name, col.width as usize, ' ')
                ));
            }
            addstr("|");
        }
        {
            // TODO is duplicate of 1st block
            label("+", 6, 4, TEXT_PAIR);
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
        label(&format!("{}", num_col_size), 20, 100, TEXT_PAIR);
        for (row_num, row) in self.data.iter().enumerate() {
            // freak out if row longer than schema

            let pair: i16 = if row_num == self.curr as usize {
                INV_TEXT_PAIR
            } else {
                TEXT_PAIR
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
                            NumMode::RELATIVE => (row_num as i32 - self.curr).abs() as usize,
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
            label("+", 7 + self.data.len() as i32, 4, TEXT_PAIR);
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
                TEXT_PAIR,
            );
        }
    }

    fn up(&mut self, by: i32) {
        if self.curr - by >= 0 {
            self.curr -= by;
        } else {
            self.curr = 0;
        }
    }

    fn down(&mut self, by: i32) {
        if self.curr + by < self.data.len() as i32 {
            self.curr += by;
        } else {
            self.curr = self.data.len() as i32 - 1;
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
    init_pair(TEXT_PAIR, COLOR_WHITE, COLOR_BLACK);
    init_pair(INV_TEXT_PAIR, COLOR_BLACK, COLOR_WHITE);

    let schema: Vec<Column> = vec![
        Column {
            name: "CSE 112".to_string(),
            width: 20,
        },
        Column {
            name: "CSE 120".to_string(),
            width: 20,
        },
    ];
    let data: Vec<Vec<String>> = vec![
        vec!["MUSC 80M".to_string(), "Discussion 1".to_string()],
        vec!["CSE 112".to_string(), "Assignment 3".to_string()],
        vec!["CSE 120".to_string(), "Midterm 2".to_string()],
        vec!["MUSC 80M".to_string(), "Discussion 1".to_string()],
        vec!["CSE 112".to_string(), "Assignment 3".to_string()],
        vec!["CSE 120".to_string(), "Midterm 2".to_string()],
        vec!["MUSC 80M".to_string(), "Discussion 1".to_string()],
        vec!["CSE 112".to_string(), "Assignment 3".to_string()],
        vec!["CSE 120".to_string(), "Midterm 2".to_string()],
        vec!["MUSC 80M".to_string(), "Discussion 1".to_string()],
        vec!["CSE 112".to_string(), "Assignment 3".to_string()],
        vec!["CSE 120".to_string(), "Midterm 2".to_string()],
        vec!["MUSC 80M".to_string(), "Discussion 1".to_string()],
        vec!["CSE 112".to_string(), "Assignment 3".to_string()],
        vec!["CSE 120".to_string(), "Midterm 2".to_string()],
    ];

    let mut table: Table = Table {
        title: "Spring 2021 Schedule".to_string(),
        subtitle: "Spring 2021 Schedule Subtitle".to_string(),
        schema,
        data,
        curr: 0,
        num_mode: NumMode::ABSOLUTE,
    };

    let mut quit = false;
    while !quit {
        erase();

        table.draw_title();
        table.draw_subtitle();
        table.draw_views();
        table.draw_headers();
        table.draw_data();
        table.draw_footer();

        let key = getch();
        match key as u8 as char {
            'q' => quit = true,
            'w' => todo!(),
            'j' => table.down(1),
            'k' => table.up(1),
            'J' => table.down(10),
            'K' => table.up(10),
            'h' => todo!(),
            'l' => todo!(),
            'v' => todo!(),
            'V' => todo!(),
            'n' => table.switch_num_mode(),
            'i' => todo!(),
            's' => todo!(),
            'f' => todo!(),
            'u' => todo!(),
            '\n' => todo!(),
            ':' => todo!(),
            '=' => todo!(),
            '?' => todo!(),
            _ => {}
        }
    }

    endwin();
}
