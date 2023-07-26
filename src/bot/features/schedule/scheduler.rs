use std::collections::HashMap;

use tokio_cron_scheduler::JobScheduler;
use uuid::Uuid;

use crate::database::repository::AsyncRepository;

pub struct Scheduler {
    inner: JobScheduler,
    ids_to_uids: HashMap<String, Uuid>,
    repository: AsyncRepository
}