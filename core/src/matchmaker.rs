use uuid::Uuid;
use crate::queue_entry;

/// A trait for implementing matchmaking algorithms.
///
/// The `Matchmaker` trait defines the interface for matchmaking systems that can
/// group queue entries (teams or players) into balanced teams for matches. Implementations
/// of this trait can use different algorithms to create fair and balanced teams
/// based on the specific requirements of the game or application.
///
/// # Type Parameters
///
/// * `T` - A type that implements the `QueueEntry` trait, representing a team or player
///         in the matchmaking queue.
///
/// # Examples
///
/// ```
/// use matchmaker::Matchmaker;
/// use matchmaker::queue_entry::QueueEntry;
/// 
/// // Implement a simple matchmaker that pairs teams randomly
/// struct RandomMatchmaker;
/// 
/// impl<T: QueueEntry> Matchmaker<T> for RandomMatchmaker {
///     fn matchmake(&self, teams: &Vec<T>) -> Result<Vec<Vec<uuid::Uuid>>, String> {
///         // Implementation details...
///     }
/// }
/// ```
pub trait Matchmaker<T> where T : queue_entry::QueueEntry {
    /// Creates teams for matches from the provided queue entries.
    ///
    /// This method takes a vector of queue entries (teams or players) and attempts to
    /// group them into balanced teams according to the matchmaking algorithm's rules.
    ///
    /// # Parameters
    ///
    /// * `teams` - A reference to a vector of queue entries to be matched.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Vec<Uuid>>)` - A vector where each inner vector represents a team for a match,
    ///   containing the Ids of the entries in that team.
    /// * `Err(String)` - An error message if matchmaking fails, explaining the reason.
    ///
    /// # Errors
    ///
    /// This method may return an error if:
    /// - There are not enough players to form valid teams
    /// - The team sizes are incompatible with the matchmaking rules
    /// - Other algorithm-specific constraints cannot be satisfied
    fn matchmake(&self, teams: &Vec<T>) -> Result<Vec<Vec<Uuid>>, String>;
}
