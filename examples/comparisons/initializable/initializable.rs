enum InitState {
    Uninitialized,
    Initializing,
    Initialized(u64),
    Disabled,
}
impl InitState {
    fn is_initializing(self) -> bool {
        match self {
            InitState::Uninitialized => false,
            InitState::Initializing => true,
            InitState::Initialized(_) => false,
            InitState::Disabled => false,
        }
    }

    fn get_initialized_version(self) -> Option<u64> {
        match self {
            InitState::Uninitialized => None,
            InitState::Initializing => None,
            InitState::Initialized(version) => Some(version),
            InitState::Disabled => None,
        }
    }

    fn is_uninitialized(self) -> bool {
        match self {
            InitState::Uninitialized => true,
            InitState::Initializing => false,
            InitState::Initialized(_) => false,
            InitState::Disabled => false,
        }
    }

    fn is_disabled(self) -> bool {
        match self {
            InitState::Uninitialized => false,
            InitState::Initializing => false,
            InitState::Initialized(_) => false,
            InitState::Disabled => true,
        }
    }

    fn can_reinitialize(self, version: u64) -> bool {
        match self {
            InitState::Uninitialized => false,
            InitState::Initializing => true,
            InitState::Initialized(old_version) => old_version < version
            InitState::Disabled => false,
        }
    }
}

contract Initializable {
    init_state: InitState;
}
impl Initializable {
    fn initialize(self)
        modifies(self.init_state)
    {
        if (!self.init_state.is_uninitialized() && !self.init_state.is_initializing()) {
            EVM::revert(InvalidInitialization());
        }

        self.init_state = InitState::Initializing;
        _;
        self.init_state = InitState::Initialized(1);

        if (isTopLevelCall) {
            EVM::emit(Initialized(1));
        }
    }

    fn reinitialize(self, version: u64)
        modifies(self.init_state)
    {
        if (self.init_state.can_reinitialize(version)) {
            EVM::revert(InvalidInitialization());
        }

        self.init_state = InitState::Initializing;
        _;
        self.init_state = InitState::Initialized(version);

        EVM::emit(Initialized(version));
    }

    fn require_initializing(self) {
        if (!self.init_state.is_initializing()) {
            EVM::revert(NotInitializing());
        }
    }

    fn disable_initializers(self)
        modifies(self.init_state)
    {
        if (self.init_state.is_initializing()) {
            EVM::revert(InvalidInitialization());
        }
        if (!self.init_state.is_disabled()) {
            self.init_state = InitState::Disabled;
            EVM::emit(Initialized(u64::MAX));
        }
    }
}

/**
 * @dev The contract is already initialized.
 */
error InvalidInitialization();

/**
 * @dev The contract is not initializing.
 */
error NotInitializing();

/**
 * @dev Triggered when the contract has been initialized or reinitialized.
 */
event Initialized(uint64 version);
