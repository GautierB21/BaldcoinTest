use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
    program_pack::{Pack, IsInitialized},
};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct BaldcoinAccount {
    pub is_initialized: bool,
    pub owner: Pubkey,
    pub balance: u64,
}

impl BaldcoinAccount {
    pub const LEN: usize = 1 + 32 + 8; // is_initialized + pubkey + balance
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum BaldcoinInstruction {
    InitializeAccount,
    Transfer {
        amount: u64,
    },
}

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = BaldcoinInstruction::try_from_slice(instruction_data)?;

    match instruction {
        BaldcoinInstruction::InitializeAccount => {
            msg!("Instruction: Initialize Account");
            process_initialize_account(program_id, accounts)
        }
        BaldcoinInstruction::Transfer { amount } => {
            msg!("Instruction: Transfer");
            process_transfer(program_id, accounts, amount)
        }
    }
}

pub fn process_initialize_account(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let account = next_account_info(account_info_iter)?;
    let owner = next_account_info(account_info_iter)?;

    if account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    let mut account_data = BaldcoinAccount::try_from_slice(&account.data.borrow())?;
    if account_data.is_initialized {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    account_data.is_initialized = true;
    account_data.owner = *owner.key;
    account_data.balance = 0;

    account_data.serialize(&mut *account.data.borrow_mut())?;
    Ok(())
}

pub fn process_transfer(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let from_account = next_account_info(account_info_iter)?;
    let to_account = next_account_info(account_info_iter)?;
    let owner = next_account_info(account_info_iter)?;

    if from_account.owner != program_id || to_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    if !owner.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut from_data = BaldcoinAccount::try_from_slice(&from_account.data.borrow())?;
    let mut to_data = BaldcoinAccount::try_from_slice(&to_account.data.borrow())?;

    if from_data.owner != *owner.key {
        return Err(ProgramError::IllegalOwner);
    }

    if from_data.balance < amount {
        return Err(ProgramError::InsufficientFunds);
    }

    from_data.balance -= amount;
    to_data.balance += amount;

    from_data.serialize(&mut *from_account.data.borrow_mut())?;
    to_data.serialize(&mut *to_account.data.borrow_mut())?;

    Ok(())
}