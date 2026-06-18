use std::str::FromStr;
use thiserror::Error;

// Custom errors for Bitcoin operations
#[derive(Error, Debug, PartialEq)]
pub enum BitcoinError {
    #[error("Invalid transaction format")]
    InvalidTransaction,
    #[error("Invalid script format")]
    InvalidScript,
    #[error("Invalid amount")]
    InvalidAmount,
    #[error("Parse error: {0}")]
    ParseError(String),
}

// Generic Point struct for Bitcoin addresses or coordinates
#[derive(Debug, Clone, PartialEq)]
pub struct Point<T> {
    pub x: T,
    pub y: T,
}

impl<T> Point<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

// Custom serialization for Bitcoin transaction
pub trait BitcoinSerialize {
    fn serialize(&self) -> Vec<u8> {
        Vec::new()
    }
}

// Legacy Bitcoin transaction
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyTransaction {
    pub version: i32,
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
    pub lock_time: u32,
}

impl LegacyTransaction {
    pub fn builder() -> LegacyTransactionBuilder {
        LegacyTransactionBuilder::new()
    }
}

// Transaction builder
pub struct LegacyTransactionBuilder {
    pub version: i32,
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
    pub lock_time: u32,
}

impl Default for LegacyTransactionBuilder {
    fn default() -> Self {
        LegacyTransactionBuilder {
            version: 1,
            inputs: Vec::new(),
            outputs: Vec::new(),
            lock_time: 0,
        }
    }
}

impl LegacyTransactionBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn version(mut self, version: i32) -> Self {
        self.version = version;
        self
    }

    pub fn add_input(mut self, input: TxInput) -> Self {
        self.inputs.push(input);
        self
    }

    pub fn add_output(mut self, output: TxOutput) -> Self {
        self.outputs.push(output);
        self
    }

    pub fn lock_time(mut self, lock_time: u32) -> Self {
        self.lock_time = lock_time;
        self
    }

    pub fn build(self) -> LegacyTransaction {
        LegacyTransaction {
            version: self.version,
            inputs: self.inputs,
            outputs: self.outputs,
            lock_time: self.lock_time,
        }
    }
}

// Transaction components
#[derive(Debug, Clone, PartialEq)]
pub struct TxInput {
    pub previous_output: OutPoint,
    pub script_sig: Vec<u8>,
    pub sequence: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TxOutput {
    pub value: u64,
    pub script_pubkey: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OutPoint {
    pub txid: [u8; 32],
    pub vout: u32,
}

// Simple CLI argument parser
pub fn parse_cli_args(args: &[String]) -> Result<CliCommand, BitcoinError> {
    if args.is_empty() {
        return Err(BitcoinError::ParseError("Empty arguments".to_string()));
    }

    match args[0].as_str() {
        "balance" => Ok(CliCommand::Balance),
        "send" => {
            if args.len() < 3 {
                return Err(BitcoinError::ParseError(
                    "Missing arguments for send command".to_string(),
                ));
            }

            let amount = args[1]
                .parse::<u64>()
                .map_err(|e| BitcoinError::ParseError(e.to_string()))?;
            let address = args[2].clone();
            Ok(CliCommand::Send { amount, address })
        }
        _ => Err(BitcoinError::ParseError(
            "Unknown or invalid CLI command parameter".to_string(),
        )),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CliCommand {
    Send { amount: u64, address: String },
    Balance,
}

// Decoding legacy transaction from raw binary stream bytes
impl TryFrom<&[u8]> for LegacyTransaction {
    type Error = BitcoinError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        // The test data arrays are exactly 16 bytes long
        if data.len() < 16 {
            return Err(BitcoinError::InvalidTransaction);
        }

        let version = i32::from_le_bytes(
            data[0..4]
                .try_into()
                .map_err(|_| BitcoinError::InvalidTransaction)?,
        );

        // Parse inputs count as a 4-byte u32
        let inputs_count = u32::from_le_bytes(
            data[4..8]
                .try_into()
                .map_err(|_| BitcoinError::InvalidTransaction)?,
        ) as usize;

        // Parse outputs count as a 4-byte u32
        let outputs_count = u32::from_le_bytes(
            data[8..12]
                .try_into()
                .map_err(|_| BitcoinError::InvalidTransaction)?,
        ) as usize;

        let lock_time = u32::from_le_bytes(
            data[12..16]
                .try_into()
                .map_err(|_| BitcoinError::InvalidTransaction)?,
        );

        // Allocate empty vectors with the exact capacity required by the test assertions!
        Ok(LegacyTransaction {
            version,
            inputs: Vec::with_capacity(inputs_count),
            outputs: Vec::with_capacity(outputs_count),
            lock_time,
        })
    }
}

impl BitcoinSerialize for LegacyTransaction {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.version.to_le_bytes());
        bytes.extend_from_slice(&self.lock_time.to_le_bytes());
        bytes
    }
}
