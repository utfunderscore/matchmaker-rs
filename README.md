<h1 align="center">
<img width="300px" src="images/logo.png" />

[![Contributors][contributors-shield]][contributors-url]
[![Forks][forks-shield]][forks-url]
[![Stargazers][stars-shield]][stars-url]
[![Issues][issues-shield]][issues-url]
</h1>


## <a name="table-of-contents"></a> Table of Contents

1. [Features](#features)
2. [Roadmap](#roadmap)
3. [Build](#build)
## 1. <a name="features"></a> Features <small><sup>[Top ▲](#table-of-contents)</sup></small>
- Websocket connections for efficient return of queue results
- Support for multiple different matchmaking algorithms (ML, KNN, Elo, etc.)
## 2. <a name="roadmap"></a> Roadmap <small><sup>[Top ▲](#table-of-contents)</sup></small> 
- [ ] Distributed workload
  - [ ] Define communication protocol
  - [ ] Establish Controller and worker instances
  - [ ] Distribute queue's across instances
  - [ ] Route HTTP requests to workers
- [ ] Matchmaker Algorithms
  - [x] Unrated Matchmaking (Variable Team Sizes / Number of teams)
  - [ ] Weighted Euclidean Distance
  - [ ] Outcome Prediction
    - [ ] Logistic Regression
    - [ ] XGBoost
    - [ ] SVM
    - [ ] Neural Networks

[contributors-shield]: https://img.shields.io/github/contributors/utfunderscore/matchmaker-rs.svg
[contributors-url]: https://github.com/utfunderscore/matchmaker-rs/graphs/contributors
[forks-shield]: https://img.shields.io/github/forks/utfunderscore/matchmaker-rs.svg
[forks-url]: https://github.com/utfunderscore/matchmaker-rs/network/members
[stars-shield]: https://img.shields.io/github/stars/utfunderscore/matchmaker-rs.svg
[stars-url]: https://github.com/utfunderscore/matchmaker-rs/stargazers
[issues-shield]: https://img.shields.io/github/issues/utfunderscore/matchmaker-rs.svg
[issues-url]: https://github.com/utfunderscore/matchmaker-rs/issues