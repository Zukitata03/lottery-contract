use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid funds")]
    InvalidFunds {},

    #[error("Contract is paused")]
    Paused {},

    #[error("Round not ended yet")]
    RoundNotEnded {},

    #[error("No participants in this round")]
    NoParticipants {},

    #[error("cant find this id ")]
    ParticipantNotFound {},
    

}

impl From<ContractError> for StdError {
    fn from(err: ContractError) -> Self {
        StdError::generic_err(err.to_string())
    }
}