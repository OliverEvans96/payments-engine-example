use std::borrow::{Borrow,BorrowMut};
use crate::types::{Account, Deposit, Withdrawal};

pub struct LockedAccount<'a>(&'a mut Account);
pub struct UnlockedAccount<'a>(&'a mut Account);

// impl<'a> UnlockedAccount<'a> {
//     /// Consumes the current access object and returns
//     /// an access with downgraded permissions.
//     /// NOTE: This doesn't actually lock the account,
//     /// and only affects this access instance,
//     /// not the account itself.
//     fn downgrade(self) -> LockedAccount<'a> {
//         LockedAccount(self.0)
//     }
//     // TODO: Rethink this
// }

mod private {
    use std::borrow::{Borrow, BorrowMut};
    // A bit hacky, but this is a workaround to avoid exposing
    // WrapsAccount publicly (since we don't want to grant
    // public access to the underlying account - that would
    // kind of defeat the point of the wrapper).
    // Normally, it's a warning (soon-to-be error) to expose
    // a private trait (WrapsAccount)
    // in a public interface (BaseAccountFeatures)
    // See https://github.com/rust-lang/rust/issues/34537
    use super::Account;
    pub trait WrapsAccount<'a, R: Borrow<Account> + 'a, M: BorrowMut<Account> + 'a> {
        fn get_account(&'a self) -> R;
        fn get_mut_account(&'a mut self) -> M;
    }
}

impl<'a> private::WrapsAccount<'a, &'a Account, &'a mut Account> for LockedAccount<'a> {
    #[inline]
    fn get_account(&'a self) -> &'a Account {
        &self.0
    }

    #[inline]
    fn get_mut_account(&mut self) -> &mut Account {
        &mut self.0
    }
}

impl<'a> private::WrapsAccount<'a, &'a Account, &'a mut Account> for UnlockedAccount<'a> {
    #[inline]
    fn get_account(&self) -> &Account {
        &self.0
    }

    #[inline]
    fn get_mut_account(&mut self) -> &mut Account {
        &mut self.0
    }
}

pub trait BaseAccountFeatures<'a, R: Borrow<Account> + 'a, M: BorrowMut<Account> + 'a>: private::WrapsAccount<'a, R, M> {
    fn modify_balances_for_dispute(&'a mut self, disputed_deposit: &Deposit) {
        let mut account = self.get_mut_account();
        let ref_account: &mut Account = account.borrow_mut();
        ref_account.available -= disputed_deposit.amount;
        ref_account.held += disputed_deposit.amount;
    }
    fn modify_balances_for_resolve(&'a mut self, disputed_deposit: &Deposit) {
        let mut account = self.get_mut_account();
        let ref_account: &mut Account = account.borrow_mut();
        ref_account.available += disputed_deposit.amount;
        ref_account.held -= disputed_deposit.amount;
    }
    fn modify_balances_for_chargeback(&'a mut self, disputed_deposit: &Deposit) {
        let mut account = self.get_mut_account();
        let ref_account: &mut Account = account.borrow_mut();
        ref_account.held -= disputed_deposit.amount;
    }
    fn view(&'a self) -> R {
        self.get_account()
    }
}

pub trait UnlockedAccountFeatures<'a, R: Borrow<Account> + 'a, M: BorrowMut<Account> + 'a>:
    private::WrapsAccount<'a, R, M>
{
    fn modify_balances_for_deposit(&'a mut self, deposit: &Deposit) {
        self.get_mut_account().borrow_mut().available += deposit.amount;
    }
    fn modify_balances_for_withdrawal(&'a mut self, withdrawal: &Withdrawal) {
        self.get_mut_account().borrow_mut().available -= withdrawal.amount;
    }
    fn lock(&'a mut self) {
        self.get_mut_account().borrow_mut().locked = true;
    }
}

impl<'a, > BaseAccountFeatures<'a, &'a Account, &'a mut Account> for LockedAccount<'a> {}
impl<'a, > BaseAccountFeatures<'a, &'a Account, &'a mut Account> for UnlockedAccount<'a> {}
impl<'a, > UnlockedAccountFeatures<'a, &'a Account, &'a mut Account> for UnlockedAccount<'a> {}

impl Account {
    pub fn access<'a>(&'a mut self) -> AccountAccess<'a> {
        if self.locked {
            AccountAccess::Locked(LockedAccount(self))
        } else {
            AccountAccess::Unlocked(UnlockedAccount(self))
        }
    }
}
pub enum AccountAccess<'a> {
    Locked(LockedAccount<'a>),
    Unlocked(UnlockedAccount<'a>),
}

impl<'a> AccountAccess<'a> {
    /// Consume the access and return a reference to the contained
    /// account wrapper, providing only the base account features.
    pub fn inner(self) -> Box<dyn BaseAccountFeatures<'a, &'a Account, &'a mut Account> + 'a> {
        match self {
            AccountAccess::Locked(account) => Box::new(account),
            AccountAccess::Unlocked(account) => Box::new(account),
        }
    }
}

impl<'a> private::WrapsAccount<'a, &'a Account, &'a mut Account> for AccountAccess<'a> {
    fn get_account(&self) -> &'a Account {
        self.inner().get_account()
    }
    fn get_mut_account(&mut self) -> &'a mut Account {
        self.inner().get_mut_account()
    }
}

impl<'a> BaseAccountFeatures<'a, &'a Account, &'a mut Account> for AccountAccess<'a> {}
