use crate::database::repository::AsyncRepository;

use super::scheduler::Scheduler;

pub async fn create_scheduler(
    bot: teloxide::Bot,
    repository: AsyncRepository,
) -> anyhow::Result<Scheduler> {
    Scheduler::new(bot, repository).await
}
