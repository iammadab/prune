# Search Notes

## Minimax (collapse model)
- You list all the moves you can play.
- For each move, you collapse the opponent's responses by assuming they'll pick the reply that hurts you the most.
- That collapse gives you a single score for that move (the worst case).
- You then pick the move whose collapsed score is best for you.

## Alpha-beta (collapse model)
- You still collapse each move by looking at the opponent's best response.
- But while collapsing a move, if you already see that this move gets worse than a different move you already have, you stop collapsing early.
- Because once a move is guaranteed to be worse than your current best, you'll never pick it.
- So alpha-beta is the same decision, just faster because you skip useless collapsing work.
- While collapsing the opponent's responses, each side tracks the best line it has found so far.
- If a line is provably worse than that best line, you stop collapsing it early.
- So alpha-beta doesn't change the decision, it just skips collapse work that can't affect the choice.
- In alpha-beta, the number you get back isn't always the real collapsed score.
- Sometimes you stop early, so the result only says “this move can't be better than what I already have.”
- That means a move can look tied when it's actually worse.

## Quiescence search (quieting the leaf)

### The problem (horizon effect)
- A fixed depth cut-off can stop the search in the middle of a tactic.
- The leaf position might look good or bad only because the tactical line wasn’t fully resolved.
- That mis-evaluation is the horizon effect: you hit the “horizon” before the position becomes stable.

### Noisy vs quiet positions
- A quiet position has no immediate tactical swings pending (no obvious captures, checks, or promotions).
- A noisy position has unresolved tactics that can quickly change the evaluation (captures, checks, promotions).

### The process
- At the normal depth limit, don’t evaluate immediately.
- First take a baseline evaluation of the current position.
- Then explore only noisy moves, continuing until the position becomes quiet.
- Use the best value found as the leaf score.
