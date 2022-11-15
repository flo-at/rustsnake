trait RandomNumberEngine {
    /// Type of the generated values.
    type ResultType;
    /// Type of the generator state.
    type StateType;

    /// Constructs the engine with a default or optionally given seed.
    fn new(seed: Option<Self::StateType>) -> Self;
    /// Sets the current state of the engine.
    fn seed(&mut self, seed: Option<Self::StateType>);
    /// Gets the generator state.
    fn state(&self) -> Self::StateType;
    /// Advances the engine's state and returns the generated value.
    fn get(&mut self) -> Self::ResultType;
    /// Advances the engine's state by \a z steps.
    fn discard(&mut self, z: usize);
    /// Gets the smallest possible value in the output range.
    const MIN: Self::ResultType = Self::ResultType::MIN;
    /// Gets the largest possible value in the output range.
    const MAX: Self::ResultType = Self::ResultType::MAX;
}

struct PCG32Fast {
    state: u64, //Self::StateType, TODO
}

impl RandomNumberEngine for PCG32Fast {
    type ResultType = u32;
    type StateType = u64;

    fn new(seed: Option<Self::StateType>) -> Self {
        Self {
            state: seed.unwrap_or(Self::DEFAULT_SEED) | Self::StateType(3),
        }
    }

    fn seed(&mut self, seed: Option<Self::StateType>) {
        self.state = seed.unwrap_or(Self::DEFAULT_SEED) | Self::StateType(3);
        // TODO deduplicate
    }

    fn state(&self) -> Self::StateType {
        self.state
    }

    fn get(&mut self) -> Self::ResultType {
        let old_state = self.state;
        self.advance();
        self.output(old_state)
    }

    fn discard(&mut self, z: usize) {
        self.state *= 1; // TODO ::masheen::utils::helpers::integral_pow_overflow(Self::MULTIPLIER, z);
    }
}

impl PCG32Fast {
    const MULTIPLIER: u64 = 6364136223846793005;
    /// Default generator seed.
    const DEFAULT_SEED: Self::StateType = 0xcafef00dd15ea5e5;

    fn advance(&mut self) {
        self.state *= Self::MULTIPLIER;
    }

    // XSH RS -- high xorshift, followed by a random shift
    const fn output(state: Self::StateType) -> Self::ResultType {
        const bits: u8 = Self::StateType::BITS;
        const xtypebits: u8 = Self::ResultType::BITS;
        const sparebits: u8 = bits - xtypebits;
        const opbits: u8 = if sparebits - 5 >= 64 {
            5
        } else if sparebits - 4 >= 32 {
            4
        } else if sparebits - 3 >= 16 {
            3
        } else if sparebits - 2 >= 4 {
            2
        } else if sparebits - 1 >= 1 {
            1
        } else {
            0
        };
        const mask: u8 = (1 << opbits) - 1;
        const maxrandshift: u8 = mask;
        const topspare: u8 = opbits;
        const bottomspare: u8 = sparebits - topspare;
        const xshift: u8 = topspare + (xtypebits + maxrandshift) / 2;
        let rshift: u8 = if opbits {
            (state >> (bits - opbits)) as u8 & mask
        } else {
            0
        };
        (state ^ (state >> xshift)) >> (bottomspare - maxrandshift + rshift)
    }
}

impl std::cmp::PartialEq for PCG32Fast {
    /// Compares the internal states of two pseudo-random number engines.
    fn eq(&self, other: &Self) -> bool {
        self.state == other.state
    }
}

// bool wrapped() const noexcept {
//     return state == 3;
// }
