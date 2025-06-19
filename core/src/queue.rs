use crate::matchmaker;
use crate::queue_entry::QueueEntry;
use matchmaker::Matchmaker;
use std::collections::HashMap;
use std::hash::Hash;
use uuid::Uuid;

/// A queue that manages teams and performs matchmaking.
///
/// A queue is a collection of queue entries (teams or players) that are to be matched
/// together base
///
/// # Type Parameters
///
/// * `T` - A type that implements the `QueueEntry` trait, representing a team
///         or player in the queue. Must also implement `Hash`, `Eq`, and `Clone`.
///
/// # Examples
///
/// ```
/// use matchmaker::Queue;
/// use matchmaker::matchmaker::Matchmaker;
/// use matchmaker::queue_entry::QueueEntry;
/// 
/// // Create a queue with a specific matchmaker implementation
/// let matchmaker = Box::new(SomeMatchmaker::new());
/// let mut queue = Queue::new("ranked_queue".to_string(), matchmaker);
/// 
/// // Add teams to the queue
/// queue.add_team(some_team);
/// 
/// // Perform matchmaking
/// let matches = queue.matchmake().unwrap();
/// ```
pub struct Queue<T: ?Sized> {
    name: String,
    teams: HashMap<Uuid, Box<T>>,
    matchmaker: Box<dyn Matchmaker<T>>,
}

impl<T> Queue<T>
where
    T: QueueEntry + Hash + Eq + Clone,
{
    /// Creates a new queue with the specified name and matchmaker.
    ///
    /// # Parameters
    ///
    /// * `name` - A string that identifies the queue
    /// * `matchmaker` - A boxed implementation of the `Matchmaker` trait that will be used
    ///                  to create matches between teams in this queue
    ///
    /// # Returns
    ///
    /// A new `Queue` instance with an empty collection of teams.
    pub fn new(name: String, matchmaker: Box<dyn Matchmaker<T>>) -> Self {
        Queue {
            name,
            teams: HashMap::new(),
            matchmaker,
        }
    }

    /// Adds a team to the queue.
    ///
    /// # Parameters
    ///
    /// * `team` - The team to add to the queue, which must implement the `QueueEntry` trait
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the team was successfully added to the queue
    /// * `Err(String)` - If the team could not be added, with an error message explaining why
    ///
    /// # Errors
    ///
    /// This method will return an error if a team with the same ID already exists in the queue.
    pub fn add_team(&mut self, team: T) -> Result<(), String> {
        if self.teams.contains_key(&team.id()) {
            return Err("Team already exists in the queue".to_string());
        }
        self.teams.insert(team.id(), Box::new(team));
        Ok(())
    }

    /// Removes a team from the queue.
    ///
    /// # Parameters
    ///
    /// * `team` - A reference to the team to remove from the queue
    ///
    /// # Returns
    ///
    /// * `Some(T)` - The removed team, if it was found in the queue
    /// * `None` - If no team with the specified ID was found in the queue
    pub fn remove_team(&mut self, team_id: Uuid) -> Option<T> {
        self.teams.remove(&team_id).map(|team| *team)
    }

    /// Performs matchmaking on the teams in the queue.
    ///
    /// This method uses the matchmaker implementation provided during queue creation
    /// to group teams into balanced matches. Teams that are successfully matched
    /// are removed from the queue.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Vec<T>>)` - A vector where each inner vector represents a team for a match,
    ///   containing the actual team objects that were matched
    /// * `Err(String)` - An error message if matchmaking fails, explaining the reason
    ///
    /// # Errors
    ///
    /// This method may return an error if the matchmaker implementation fails to create
    /// valid matches, for reasons such as:
    /// - There are not enough teams in the queue
    /// - The team compositions are incompatible with the matchmaking rules
    /// - Other algorithm-specific constraints cannot be satisfied
    pub fn matchmake(&mut self) -> Result<Vec<Vec<T>>, String> {
        let teams: Vec<Box<T>> = self.teams.values().into_iter().cloned().collect();

        let teams = self.matchmaker.matchmake(&teams)?;

        let teams = teams
            .into_iter()
            .map(|team_ids| {
                team_ids
                    .into_iter()
                    .filter_map(|id| self.teams.remove(&id).map(|x| *x))
                    .collect::<Vec<T>>()
            })
            .collect::<Vec<Vec<T>>>();

        Ok(teams)
    }
}
