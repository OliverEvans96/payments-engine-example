use crate::types::{Deposit, Withdrawal, Account};

pub fn lock_account(account: &mut Account) {
    account.locked = true;
}

pub fn modify_balances_for_deposit(deposit: &Deposit, account: &mut Account) {
    account.available += deposit.amount;
}

pub fn modify_balances_for_withdrawal(withdrawal: &Withdrawal, account: &mut Account) {
    account.available -= withdrawal.amount;
}

pub fn modify_balances_for_dispute(disputed_deposit: &Deposit, account: &mut Account) {
    account.available -= disputed_deposit.amount;
    account.held += disputed_deposit.amount;
}

pub fn modify_balances_for_resolve(disputed_deposit: &Deposit, account: &mut Account) {
    account.available += disputed_deposit.amount;
    account.held -= disputed_deposit.amount;
}

pub fn modify_balances_for_chargeback(disputed_deposit: &Deposit, account: &mut Account) {
    account.held -= disputed_deposit.amount;
}