#[derive(PartialEq, Eq)]
pub enum NSKeys {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,

    Num0,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,

    Apostrophe,
    BackApostrophe,
    Backslash,
    Backspace,
    CapsLock,
    Comma,
    Delete,
    Equals,
    Escape,
    FrontSlash,
    LeftBracket,
    Minus,
    Period,
    Return,
    RightBracket,
    Semicolon,
    Tab,
    Space,

    Shift,
    Control,
    Command,
    Option,
    Fn,

    Up,
    Left,
    Down,
    Right,

    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,

    Home,
    End,
    PageUp,
    PageDown,
    Insert,
}

impl NSKeys {
    /// sourced from here:
    /// https://github.com/phracker/MacOSX-SDKs/blob/master/MacOSX10.6.sdk/System/Library/Frameworks/Carbon.framework/Versions/A/Frameworks/HIToolbox.framework/Versions/A/Headers/Events.h
    pub fn from(key_code: u16) -> Option<Self> {
        use NSKeys::*;
        let key = match key_code {
            0 => A,
            1 => S,
            2 => D,
            3 => F,
            4 => H,
            5 => G,
            6 => Z,
            7 => X,
            8 => C,
            9 => V,
            11 => B,
            12 => Q,
            13 => W,
            14 => E,
            15 => R,
            16 => Y,
            17 => T,
            18 => Num1,
            19 => Num2,
            20 => Num3,
            21 => Num4,
            22 => Num6,
            23 => Num5,
            24 => Equals,
            25 => Num9,
            26 => Num7,
            27 => Minus,
            28 => Num8,
            29 => Num0,
            30 => RightBracket,
            31 => O,
            32 => U,
            33 => LeftBracket,
            34 => I,
            35 => P,
            36 => Return,
            37 => L,
            38 => J,
            39 => Apostrophe,
            40 => K,
            41 => Semicolon,
            42 => Backslash,
            43 => Comma,
            44 => FrontSlash,
            45 => N,
            46 => M,
            47 => Period,
            48 => Tab,
            49 => Space,
            50 => BackApostrophe,
            51 => Backspace,
            53 => Escape,
            55 => Command,
            56 => Shift,
            57 => CapsLock,
            58 => Option,
            59 => Control,
            60 => Shift,   // Right
            61 => Option,  // Right
            62 => Control, // Right
            63 => Fn,
            64 => F17,
            65 => Period, // Keypad

            79 => F18,
            80 => F19,
            81 => Equals, // Keypad
            82 => Num0,   // Keypad
            83 => Num1,   // Keypad
            84 => Num2,   // Keypad
            85 => Num3,   // Keypad
            86 => Num4,   // Keypad
            87 => Num5,   // Keypad
            88 => Num6,   // Keypad
            89 => Num7,   // Keypad
            90 => F20,
            91 => Num8, // Keypad
            92 => Num9, // Keypad
            96 => F5,
            97 => F6,
            98 => F7,
            99 => F3,
            100 => F8,
            101 => F9,
            103 => F11,
            105 => F13,
            106 => F16,
            107 => F14,
            109 => F10,
            111 => F12,
            113 => F15,
            115 => Home,
            116 => PageUp,
            118 => F4,
            119 => End,
            120 => F2,
            121 => PageDown,
            122 => F1,
            123 => Left,
            124 => Right,
            125 => Down,
            126 => Up,
            _ => return None,
        };

        // esoteric, ignored for now

        // 67 => KeypadMultiply,
        // 69 => KeypadPlus,
        // 71 => KeypadClear,
        // 72 => VolumeUp,
        // 73 => VolumeDown,
        // 74 => Mute,
        // 75 => KeypadDivide,
        // 76 => KeypadEnter,
        // 78 => KeypadMinus,
        // 114 => Help,
        // 117 => ForwardDelete,

        Some(key)
    }

    pub fn valid_text(&self) -> bool {
        use NSKeys::*;
        match self {
            A | B | C | D | E | F | G | H | I | J | K | L | M | N | O | P | Q | R | S | T | U
            | V | W | X | Y | Z | Num0 | Num1 | Num2 | Num3 | Num4 | Num5 | Num6 | Num7 | Num8
            | Num9 | Apostrophe | BackApostrophe | Backslash | Comma | Equals | FrontSlash
            | LeftBracket | Minus | Period | RightBracket | Semicolon | Space => true,

            CapsLock | Command | Control | Delete | Escape | Option | Return | Shift | Tab | Up
            | Left | Down | Right | Backspace | Fn | F1 | F2 | F3 | F4 | F5 | F6 | F7 | F8 | F9
            | F10 | F11 | F12 | F13 | F14 | F15 | F16 | F17 | F18 | F19 | F20 | Home | End
            | PageUp | PageDown | Insert => false,
        }
    }

    pub fn egui_key(&self) -> Option<egui::Key> {
        use NSKeys::*;
        let key = match self {
            A => egui::Key::A,
            B => egui::Key::B,
            C => egui::Key::C,
            D => egui::Key::D,
            E => egui::Key::E,
            F => egui::Key::F,
            G => egui::Key::G,
            H => egui::Key::H,
            I => egui::Key::I,
            J => egui::Key::J,
            K => egui::Key::K,
            L => egui::Key::L,
            M => egui::Key::M,
            N => egui::Key::N,
            O => egui::Key::O,
            P => egui::Key::P,
            Q => egui::Key::Q,
            R => egui::Key::R,
            S => egui::Key::S,
            T => egui::Key::T,
            U => egui::Key::U,
            V => egui::Key::V,
            W => egui::Key::W,
            X => egui::Key::X,
            Y => egui::Key::Y,
            Z => egui::Key::Z,
            Num0 => egui::Key::Num0,
            Num1 => egui::Key::Num1,
            Num2 => egui::Key::Num2,
            Num3 => egui::Key::Num3,
            Num4 => egui::Key::Num4,
            Num5 => egui::Key::Num5,
            Num6 => egui::Key::Num6,
            Num7 => egui::Key::Num7,
            Num8 => egui::Key::Num8,
            Num9 => egui::Key::Num9,
            Delete => egui::Key::Delete,
            Escape => egui::Key::Escape,
            Return => egui::Key::Enter,
            Tab => egui::Key::Tab,
            Left => egui::Key::ArrowLeft,
            Right => egui::Key::ArrowRight,
            Up => egui::Key::ArrowUp,
            Down => egui::Key::ArrowDown,
            Space => egui::Key::Space,
            Backspace => egui::Key::Backspace,
            F1 => egui::Key::F1,
            F2 => egui::Key::F2,
            F3 => egui::Key::F3,
            F4 => egui::Key::F4,
            F5 => egui::Key::F5,
            F6 => egui::Key::F6,
            F7 => egui::Key::F7,
            F8 => egui::Key::F8,
            F9 => egui::Key::F9,
            F10 => egui::Key::F10,
            F11 => egui::Key::F11,
            F12 => egui::Key::F12,
            F13 => egui::Key::F13,
            F14 => egui::Key::F14,
            F15 => egui::Key::F15,
            F16 => egui::Key::F16,
            F17 => egui::Key::F17,
            F18 => egui::Key::F18,
            F19 => egui::Key::F19,
            F20 => egui::Key::F20,
            Home => egui::Key::Home,
            End => egui::Key::End,
            PageUp => egui::Key::PageUp,
            PageDown => egui::Key::PageDown,
            Insert => egui::Key::Insert,
            Apostrophe | Comma | BackApostrophe | Backslash | CapsLock | Command | Control
            | Equals | FrontSlash | LeftBracket | Minus | Option | Period | RightBracket
            | Semicolon | Shift | Fn => return None,
        };

        Some(key)
    }
}