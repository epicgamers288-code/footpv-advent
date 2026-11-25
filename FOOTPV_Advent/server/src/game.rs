use rand::Rng;
use serde_json::{Value as V, json};
pub const GAMES: [(&str, &str, i32); 10] = [
    (
        "Тренер сказал: «Легко бежим»",
        "Выбери свободную дорожку, когда конусы приблизятся к линии игрока.",
        40,
    ),
    (
        "Поле не подвело",
        "Проведи от мяча к кольцу. Длина жеста задаёт силу. На льду мяч проскользит дальше.",
        30,
    ),
    (
        "Мам, а где мои бутсы?",
        "Положи нужную вещь в сумку. Нажми на подходящий предмет.",
        45,
    ),
    (
        "Судья, ну как так?!",
        "Посмотри эпизод и выбери решение по правилам футбола 11×11.",
        30,
    ),
    (
        "Папа на трибуне",
        "Прочитай ситуацию на поле и выбери действие.",
        40,
    ),
    (
        "Закрываем калитку",
        "Нажми в створе там, куда летит мяч, перед пересечением линии ворот.",
        30,
    ),
    (
        "До тренировки 5 минут",
        "Пройди от входа до золотого квадрата. Нажимай соседние клетки с открытым проходом.",
        45,
    ),
    (
        "Девяточка, родная!",
        "Зажми и отпусти над мишенью. Зелёный индикатор показывает подходящую силу.",
        30,
    ),
    (
        "Ещё одна и всё",
        "Набивай мяч точным касанием в нижней золотой зоне. Серия приносит бонус.",
        40,
    ),
    (
        "До Нового гола!",
        "Обойди конусы, отдай передачу, пройди зимний участок и заверши атаку ударом.",
        60,
    ),
];
pub fn scene(game: i32, step: i32) -> V {
    let kind = if game == 10 {
        [1, 5, 2, 8][step as usize % 4]
    } else {
        game
    };
    let mut r = rand::thread_rng();
    let mut v = json!({"kind":kind,"tip":GAMES[(kind-1)as usize].1,"x":r.gen_range(23..78),"lane":r.gen_range(0..3),"phase":r.gen_range(0..75),"speed":1.+0.035*f64::from(step.min(18))});
    let questions: &[(&str, [&str; 3])] = match kind {
        3 => &[
            (
                "Зал с паркетом. Что взять для ног?",
                ["Футзалки", "Мандарин", "Картошка"],
            ),
            (
                "Зимой мёрзнут руки. Что взять?",
                ["Перчатки", "Ёлочный шар", "Пульт"],
            ),
            (
                "Что защитит голень на матче?",
                ["Щитки", "Бутылка", "Мандарин"],
            ),
            ("Перерыв. Что взять для питья?", ["Вода", "Бутсы", "Гетры"]),
        ],
        4 => &[
            (
                "Мяч целиком пересёк боковую линию.",
                ["Аут", "Играть", "Угловой"],
            ),
            (
                "Часть мяча остаётся над боковой линией.",
                ["Играть", "Аут", "От ворот"],
            ),
            (
                "Мяч от штанги вернулся в поле. Нарушений нет.",
                ["Играть", "Угловой", "От ворот"],
            ),
        ],
        5 => &[
            (
                "Пустые ворота в двух шагах. Защитники позади.",
                ["Удар", "Навес", "Пас назад"],
            ),
            (
                "Два защитника блокируют удар. Партнёр свободен у пустых ворот.",
                ["Передача", "В аут", "В блок"],
            ),
            (
                "Впереди свободный коридор. Партнёры закрыты, до ворот далеко.",
                ["Ведение", "В аут", "Удар издали"],
            ),
        ],
        _ => &[],
    };
    if !questions.is_empty() {
        let i = r.gen_range(0..questions.len());
        let (q, mut opts) = questions[i];
        let shift = r.gen_range(0..3);
        opts.rotate_right(shift);
        v["q"] = json!(q);
        v["variant"] = json!(i);
        v["options"] = json!(opts);
        v["answer"] = json!(shift);
    }
    if kind == 7 {
        let mut seen = [false; 25];
        seen[0] = true;
        let mut stack = vec![0i32];
        let mut edges = vec![];
        while let Some(&i) = stack.last() {
            let choices: Vec<_> = [i - 5, i + 5, i - 1, i + 1]
                .into_iter()
                .filter(|&j| {
                    (0..25).contains(&j)
                        && (i % 5 - j % 5).abs() + (i / 5 - j / 5).abs() == 1
                        && !seen[j as usize]
                })
                .collect();
            if choices.is_empty() {
                stack.pop();
            } else {
                let j = choices[r.gen_range(0..choices.len())];
                edges.push([i, j]);
                seen[j as usize] = true;
                stack.push(j)
            }
        }
        v["edges"] = json!(edges);
    }
    v
}
pub fn score(c: &V, a: &V, ms: i32, power: i32) -> i32 {
    if !(160..=60000).contains(&ms) {
        return 0;
    }
    let x = a["x"].as_f64().unwrap_or(-500.);
    let y = a["y"].as_f64().unwrap_or(-500.);
    let t = f64::from(ms) * 0.001 * c["speed"].as_f64().unwrap();
    let phase = c["phase"].as_f64().unwrap() / 10.;
    let target = c["x"].as_f64().unwrap();
    let lane = c["lane"].as_i64().unwrap();
    let ok =
        match c["kind"].as_i64().unwrap() {
            1 => (0.82..2.5).contains(&t) && (x / 33.334).floor() as i64 == lane,
            2 => {
                let friction = [0.94, 1.45, 0.76][lane as usize];
                let dx = 50. + (x - 50.) * friction - target;
                let dy = 80. + (y - 80.) * friction - 38.;
                a["origin"] == true && dx * dx + dy * dy < 115.
            }
            3..=5 => a["choice"].as_i64() == c["answer"].as_i64(),
            6 => (0.65..1.9).contains(&t) && (target - x).abs() < 18.,
            7 => a["path"].as_array().is_some_and(|p| {
                p.len() <= 60
                    && p.first() == Some(&json!(0))
                    && p.last() == Some(&json!(24))
                    && ms >= 60 * p.len() as i32
                    && p.windows(2).all(|w| {
                        c["edges"].as_array().unwrap().iter().any(|e| {
                            (e[0] == w[0] && e[1] == w[1]) || (e[1] == w[0] && e[0] == w[1])
                        })
                    })
            }),
            8 => {
                let dx = x - (50. + 34. * (t + phase).sin());
                dx * dx + (y - 32.).powi(2) < 225. && (180..=900).contains(&power)
            }
            9 => {
                let f = (t * 0.65 + phase).fract();
                let h = 80. - 240. * f * (1. - f);
                (h - y).abs() < 16. && (x - target).abs() < 16. && h > 52.
            }
            _ => false,
        };
    if ok {
        100 + (3000 - ms).max(0) / 100
    } else {
        0
    }
}
pub fn reward(n: u8) -> i32 {
    match n {
        0..=39 => 1,
        40..=69 => 2,
        70..=89 => 3,
        90..=97 => 4,
        _ => 5,
    }
}
