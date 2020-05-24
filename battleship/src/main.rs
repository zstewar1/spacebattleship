use std::{
    fmt,
    io::{self, BufRead, Write},
    str,
};

use clap::{App, Arg, ArgMatches};
use once_cell::sync::Lazy;
use rand::{distributions::Uniform, Rng};
use regex::Regex;

use spacebattleship::game::simple::{
    CannotPlaceReason, Coordinate, Game, GameSetup, Orientation, Player, Ship,
};

/// Range of valid coordinates for the standard 10x10 game.
static COORD_RANGE: Lazy<Uniform<Coordinate>> =
    Lazy::new(|| Uniform::new(Coordinate::new(0, 0), Coordinate::new(10, 10)));

fn main() -> io::Result<()> {
    let matches = App::new("Battleship")
        .version("1.0")
        .author("Zachary Stewart <zachary@zstewart.com>")
        .about("Simple command line battleship game.")
        .arg(
            Arg::with_name("first_player")
                .short("f")
                .long("first_player")
                .value_name("FIRST_PLAYER")
                .help("pre-specify which player goes first")
                .takes_value(true)
                .possible_values(&["human", "me", "computer", "bot", "random", "rand"])
                .case_insensitive(true),
        )
        .get_matches();

    let stdin = std::io::stdin();
    let mut input = InputReader::new(stdin.lock());
    let mut rng = rand::thread_rng();

    let player = choose_player(&matches, &mut input)?;
    let bot = player.opponent();

    let mut setup = GameSetup::new();
    choose_placements(&mut rng, &mut setup, player, &mut input)?;
    choose_random_placements(&mut rng, &mut setup, bot);

    Ok(())
}

/// Choose which [`Player`] is the human player based on either args or cli input.
fn choose_player<B: BufRead>(
    matches: &ArgMatches,
    input: &mut InputReader<B>,
) -> io::Result<Player> {
    Ok(if let Some(clichoice) = matches.value_of("first_player") {
        match clichoice {
            "human" | "me" => Player::P1,
            "computer" | "bot" => Player::P2,
            "random" | "rand" => rand::random(),
            _ => unreachable!(),
        }
    } else {
        input.read_input_lower("Do you want to go first? (Y/n)", |input| match input {
            "yes" | "y" | "first" | "1" | "1st" | "" => Some(Player::P1),
            "no" | "n" | "second" | "2" | "2nd" => Some(Player::P2),
            _ => {
                println!("Invalid selection.");
                None
            }
        })?
    })
}

/// Choose placements for all ships using input from the player.
fn choose_placements(
    rng: &mut impl Rng,
    setup: &mut GameSetup,
    player: Player,
    input: &mut InputReader<impl BufRead>,
) -> io::Result<()> {
    enum Command {
        Done,
        Place(Ship, Coordinate, Orientation),
        Unplace(Ship),
        Clear,
        RandomizeRest,
        Help,
    }
    println!();
    println!("Place ships. Type help or ? for commands.");
    loop {
        println!();
        /// Matcher for commands with args.
        static PLACE: Lazy<Regex> = Lazy::new(|| {
            Regex::new(
                r"^(?x)(?:place|put)\s+
        (?P<ship>\w+)\s+
        (?:(?:at|on|to|->|=>)\s+)?
        (?P<x>[0-9]+)(?:\s*,\s*|\s+)(?P<y>[0-9]+)\s+
        (?P<dir>\w+)$",
            )
            .unwrap()
        });
        static UNPLACE: Lazy<Regex> = Lazy::new(|| {
            Regex::new(
                r"^(?x)(?:un-?place|remove)\s+
        (?P<ship>\w+)$",
            )
            .unwrap()
        });

        if setup.is_player_ready(player) {
            println!("All ships placed, type done to start the game");
        } else {
            let mut ships = setup.get_pending_ships(player);
            print!(
                "Remaining ships to place: {}",
                ShipFullName(ships.next().unwrap())
            );
            for ship in ships {
                print!(", {}", ShipFullName(ship));
            }
            println!();
        }
        println!("Your current board setup:");
        show_setup_board(setup, player);
        println!();

        let cmd = input.read_input_lower("> ", |input| match input {
            "?" | "help" | "h" => Some(Command::Help),
            "randomize" | "rand" | "random" => Some(Command::RandomizeRest),
            "done" | "start" => Some(Command::Done),
            "clear" => Some(Command::Clear),
            other => if let Some(captures) = PLACE.captures(other) {
                let ship = match captures.name("ship").unwrap().as_str() {
                    "cv" | "carrier" => Ship::Carrier,
                    "bb" | "battleship" => Ship::Battleship,
                    "ca" | "cl" | "cruiser" => Ship::Cruiser,
                    "ss" | "sub" | "submarine" => Ship::Submarine,
                    "dd" | "destroyer" => Ship::Destroyer,
                    other => {
                        println!("invalid ship: {}, choose \"carrier\", \"battleship\", \"cruiser\", \"submarine\", or \"destroyer\"", other);
                        return None;
                    }
                };
                let x = match captures.name("x").unwrap().as_str().parse() {
                    Err(_) => {
                        println!("invalid x: {}, must be a number in range [0,9]", captures.name("x").unwrap().as_str());
                        return None;
                    }
                    Ok(x) if x >= 10 => {
                        println!("x must be in range [0,9], got {}", x);
                        return None;
                    }
                    Ok(x) => x,
                };
                let y = match captures.name("y").unwrap().as_str().parse() {
                    Err(_) => {
                        println!("invalid y: {}", captures.name("y").unwrap().as_str());
                        return None;
                    }
                    Ok(y) if y >= 10 => {
                        println!("y must be in range [0,9], got {}", y);
                        return None;
                    }
                    Ok(y) => y,
                };
                let dir = match captures.name("dir").unwrap().as_str() {
                    "up" | "north" | "u" | "n" => Orientation::Up,
                    "down" | "south" | "d" | "s" => Orientation::Down,
                    "left" | "west" | "l" | "w" => Orientation::Left,
                    "right" | "east" | "r" | "e" => Orientation::Right,
                    other => {
                        println!("invalid direction {}, choose \"up\", \"down\", \"left\", or \"right\"", other);
                        return None;
                    }
                };
                Some(Command::Place(ship, Coordinate::new(x, y), dir))
            } else if let Some(captures) = UNPLACE.captures(other) {
                Some(Command::Unplace(match captures.name("ship").unwrap().as_str() {
                    "cv" | "carrier" => Ship::Carrier,
                    "bb" | "battleship" => Ship::Battleship,
                    "ca" | "cl" | "cruiser" => Ship::Cruiser,
                    "ss" | "sub" | "submarine" => Ship::Submarine,
                    "dd" | "destroyer" => Ship::Destroyer,
                    "all" => return Some(Command::Clear),
                    other => {
                        println!("invalid ship: {}, choose \"carrier\", \"battleship\", \"cruiser\", \"submarine\", \"destroyer\", or \"all\"", other);
                        return None;
                    }
                }))
            } else {
                println!("Invalid ship-placement command \"{}\". Use '?' for help", other);
                None
            }
        })?;

        match cmd {
            Command::Done if setup.is_player_ready(player) => break,
            Command::Done => println!("You must place all your ships first!"),
            Command::Place(ship, start, dir) => {
                if setup.get_placement(player, ship).is_some() {
                    setup.unplace_ship(player, ship);
                }
                match setup.place_ship(player, ship, start, dir) {
                    Ok(()) => {}
                    Err(CannotPlaceReason::AlreadyOccupied) => {
                        println!("Invalid placement: overlaps existing ship.");
                    }
                    Err(CannotPlaceReason::AlreadyPlaced) => unreachable!(),
                    Err(CannotPlaceReason::InsufficientSpace) => {
                        println!("Invalid placement: not enough space on the board.");
                    }
                }
            }
            Command::Unplace(ship) => {
                setup.unplace_ship(player, ship);
            }
            Command::Clear => {
                for ship in Ship::ALL {
                    setup.unplace_ship(player, *ship);
                }
            }
            Command::RandomizeRest => choose_random_placements(rng, setup, player),
            Command::Help => {
                println!(
                    "Available Commands:
    done                        if all ships are placed, start the game.
    place <ship> <x>,<y> <dir>  place the ship at the given coordinate in the given direction.
        Possible directions are \"up\", \"down\", \"left\", and \"right\". See below for ships.
    unplace <ship>              clear the placement of the specified ship.
        See below for possible ship. Additionally \"all\" may be specified to clear all placements.
    clear                       clears all ship placements.
    randomize                   randomize the placements of the remaining ships.
    
Available Ships: 
    \"carrier\" (\"cv\")
    \"battleship\" (\"bb\")
    \"cruiser\" (\"cl\")
    \"submarine\" (\"ss\")
    \"destroyer\" (\"dd\")",
                );
            }
        }
    }
    Ok(())
}

/// Choose all ship placements for all un-placed ships owned by the given player.
fn choose_random_placements(rng: &mut impl Rng, setup: &mut GameSetup, player: Player) {
    for &ship in Ship::ALL {
        loop {
            let start = rng.sample(&*COORD_RANGE);
            let dir = rng.gen();
            match setup.place_ship(player, ship, start, dir) {
                Ok(()) | Err(CannotPlaceReason::AlreadyPlaced) => break,
                _ => {}
            }
        }
    }
}

/// Print out the setup board for the given player.
fn show_setup_board(setup: &GameSetup, player: Player) {
    enum SetupCell {
        Empty,
        Ship(ShipAbbreviation),
    }
    impl fmt::Display for SetupCell {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                SetupCell::Empty => f.pad("~~"),
                SetupCell::Ship(abbrev) => fmt::Display::fmt(abbrev, f),
            }
        }
    }
    show_board(setup.iter_board(player).map(|row| {
        row.map(|cell| match cell {
            Some(ship) => SetupCell::Ship(ShipAbbreviation(ship)),
            None => SetupCell::Empty,
        })
    }))
}

/// Print out the fully-revealed board for the given player.
fn show_revealed_board(game: &Game, player: Player) {
    enum RevealedCell {
        Empty,
        Shot,
        NotShot(ShipAbbreviation),
        Hit(ShipAbbreviation),
        Sunk(ShipAbbreviation),
    }
    impl fmt::Display for RevealedCell {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                RevealedCell::Empty => f.pad("~~"),
                RevealedCell::Shot => f.pad("x"),
                RevealedCell::NotShot(ship) => fmt::Display::fmt(ship, f),
                RevealedCell::Hit(ship) => {
                    let mut buf = *b"x00";
                    buf[1..].copy_from_slice(ship.abbrev().as_bytes());
                    f.pad(str::from_utf8(&buf[..]).unwrap())
                }
                RevealedCell::Sunk(ship) => {
                    let mut buf = *b"X00";
                    buf[1..].copy_from_slice(ship.abbrev().as_bytes());
                    f.pad(str::from_utf8(&buf[..]).unwrap())
                }
            }
        }
    }
    show_board(game.iter_board(player).map(|row| {
        row.map(|cell| match cell.ship() {
            None if cell.hit() => RevealedCell::Shot,
            None => RevealedCell::Empty,
            Some(ship) if ship.sunk() => RevealedCell::Sunk(ShipAbbreviation(*ship.id())),
            Some(ship) if cell.hit() => RevealedCell::Hit(ShipAbbreviation(*ship.id())),
            Some(ship) => RevealedCell::NotShot(ShipAbbreviation(*ship.id())),
        })
    }))
}

/// Print out the obfuscated board for the given player.
fn show_obfuscated_board(game: &Game, player: Player) {
    enum HiddenCell {
        NotShot,
        Miss,
        Hit(ShipAbbreviation),
        Sunk(ShipAbbreviation),
    }
    impl fmt::Display for HiddenCell {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                HiddenCell::NotShot => f.pad("~~"),
                HiddenCell::Miss => f.pad("x"),
                HiddenCell::Hit(ship) => {
                    let mut buf = *b"x00";
                    buf[1..].copy_from_slice(ship.abbrev().as_bytes());
                    f.pad(str::from_utf8(&buf[..]).unwrap())
                }
                HiddenCell::Sunk(ship) => {
                    let mut buf = *b"X00";
                    buf[1..].copy_from_slice(ship.abbrev().as_bytes());
                    f.pad(str::from_utf8(&buf[..]).unwrap())
                }
            }
        }
    }
    show_board(game.iter_board(player).map(|row| {
        row.map(|cell| match cell.ship() {
            _ if !cell.hit() => HiddenCell::NotShot,
            None => HiddenCell::Miss,
            Some(ship) if ship.sunk() => HiddenCell::Sunk(ShipAbbreviation(*ship.id())),
            Some(ship) => HiddenCell::Hit(ShipAbbreviation(*ship.id())),
        })
    }))
}

/// Show the board by printing the grid. Takes an iterator over the rows of iterators over
/// the items
fn show_board(rows: impl Iterator<Item = impl Iterator<Item = impl fmt::Display>>) {
    print!("   ");
    for i in 0..10 {
        print!("{:^4}", i);
    }
    println!();
    for (i, row) in rows.enumerate() {
        print!("{:>2} ", i);
        for cell in row {
            print!("{:^4}", cell);
        }
        println!();
    }
}

/// Display helper that prints the ship's full name.
struct ShipFullName(Ship);

impl ShipFullName {
    fn name(&self) -> &'static str {
        match self.0 {
            Ship::Carrier => "carrier",
            Ship::Battleship => "battleship",
            Ship::Cruiser => "cruiser",
            Ship::Submarine => "submarine",
            Ship::Destroyer => "destroyer",
        }
    }
}

impl fmt::Display for ShipFullName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad(self.name())
    }
}
/// Display helper that prints the ship's type abbreviation
struct ShipAbbreviation(Ship);

impl ShipAbbreviation {
    fn abbrev(&self) -> &'static str {
        match self.0 {
            Ship::Carrier => "cv",
            Ship::Battleship => "bb",
            Ship::Cruiser => "cl",
            Ship::Submarine => "ss",
            Ship::Destroyer => "dd",
        }
    }
}

impl fmt::Display for ShipAbbreviation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad(self.abbrev())
    }
}

/// Helper to read input from the player.
struct InputReader<B> {
    read: B,
    buf: String,
}

impl<B> InputReader<B> {
    fn new(read: B) -> Self {
        Self {
            read,
            buf: String::new(),
        }
    }
}

impl<B: BufRead> InputReader<B> {
    /// Repeatedly tries to read input until the input checker returns `Some`. Converts
    /// to ascii lower before running the checker.
    fn read_input_lower<F, T>(&mut self, prompt: &str, mut checker: F) -> io::Result<T>
    where
        F: FnMut(&str) -> Option<T>,
    {
        loop {
            self.read_input_inner(prompt)?;
            self.buf.make_ascii_lowercase();
            if let Some(val) = checker(self.buf.trim()) {
                return Ok(val);
            }
        }
    }

    /// Repeatedly tries to read input until the input checker returns `Some`.
    fn read_input<F, T>(&mut self, prompt: &str, mut checker: F) -> io::Result<T>
    where
        F: FnMut(&str) -> Option<T>,
    {
        loop {
            self.read_input_inner(prompt)?;
            if let Some(val) = checker(self.buf.trim()) {
                return Ok(val);
            }
        }
    }

    /// Helper to print the prompt, clear the string buffer and read a line.
    fn read_input_inner(&mut self, prompt: &str) -> io::Result<()> {
        print!("{} ", prompt);
        io::stdout().flush()?;
        self.buf.clear();
        if self.read.read_line(&mut self.buf)? == 0 {
            println!();
            std::process::exit(0);
        }
        Ok(())
    }
}
