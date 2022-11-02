pub const note_name: [&str; 108] = [
    "C0", "C#0", "D0", "D#0", "E0", "F0", "F#0", "G0", "G#0", "A0", "A#0", "B0", "C1", "C#1", "D1",
    "D#1", "E1", "F1", "F#1", "G1", "G#1", "A1", "A#1", "B1", "C2", "C#2", "D2", "D#2", "E2", "F2",
    "F#2", "G2", "G#2", "A2", "A#2", "B2", "C3", "C#3", "D3", "D#3", "E3", "F3", "F#3", "G3",
    "G#3", "A3", "A#3", "B3", "C4", "C#4", "D4", "D#4", "E4", "F4", "F#4", "G4", "G#4", "A4",
    "A#4", "B4", "C5", "C#5", "D5", "D#5", "E5", "F5", "F#5", "G5", "G#5", "A5", "A#5", "B5", "C6",
    "C#6", "D6", "D#6", "E6", "F6", "F#6", "G6", "G#6", "A6", "A#6", "B6", "C7", "C#7", "D7",
    "D#7", "E7", "F7", "F#7", "G7", "G#7", "A7", "A#7", "B7", "C8", "C#8", "D8", "D#8", "E8", "F8",
    "F#8", "G8", "G#8", "A8", "A#8", "B8",
];

pub const note_freq: [u16; 108] = [
    16, 17, 18, 19, 21, 22, 23, 25, 26, 28, 29, 31, 33, 35, 37, 39, 41, 44, 46, 49, 52, 55, 58, 62,
    65, 69, 73, 78, 82, 87, 93, 98, 104, 110, 117, 123, 131, 139, 147, 156, 165, 175, 185, 196,
    208, 220, 233, 247, 262, 277, 294, 311, 330, 349, 370, 392, 415, 440, 466, 494, 523, 554, 587,
    622, 659, 698, 740, 784, 831, 880, 932, 988, 1047, 1109, 1175, 1245, 1319, 1397, 1480, 1568,
    1661, 1760, 1865, 1976, 2093, 2217, 2349, 2489, 2637, 2794, 2960, 3136, 3322, 3520, 3729, 3951,
    4186, 4435, 4699, 4978, 5274, 5588, 5920, 6272, 6645, 7040, 7459, 7902,
];

pub const note_c3_index: usize = 36;

pub const NOTES_PER_OCTAVE: u32 = 12;

const fn letter_to_note_num(letter: char) -> Option<usize> {
    match letter {
        'C' => Some(0),
        'D' => Some(2),
        'E' => Some(4),
        'F' => Some(5),
        'G' => Some(7),
        'A' => Some(9),
        'B' => Some(11),
        _ => None,
    }
}

pub fn note_from_string(name: &str) -> Option<usize> {
    if name.len() < 2 {
        return None;
    }

    let note_num = if let Some(letter @ 'A'..='G') = name.chars().nth(0) {
        letter_to_note_num(letter)? as u32
    } else {
        return None;
    };

    match (name.chars().nth(1), name.chars().nth(2)) {
        (Some(octave @ '0'..='8'), None) => {
            let octave_num = octave.to_digit(10)?;
            let index = usize::try_from(NOTES_PER_OCTAVE * octave_num + note_num).ok()?;
            Some(index)
        }
        (Some('#'), Some(octave @ '0'..='8')) => {
            let octave_num = octave.to_digit(10)?;
            let index = usize::try_from(NOTES_PER_OCTAVE * octave_num + note_num).ok()?;
            Some(index + 1)
        }
        _ => return None,
    }
}

pub fn note_to_render(note: usize) -> String {
    let octave = note / NOTES_PER_OCTAVE as usize;
    let letter = match note % NOTES_PER_OCTAVE as usize {
        0 => "C-",
        1 => "C#",
        2 => "D-",
        3 => "D#",
        4 => "E-",
        5 => "F-",
        6 => "F#",
        7 => "G-",
        8 => "G#",
        9 => "A-",
        10 => "A#",
        11 => "B-",
        _ => "--",
    };
    format!("{letter}{octave}")
}
