## GET UP

Get up is a simple (read barebones) tool meant to be used with a standing desk. It sends out a notification every time the timer ends, reminding the user to either sit or stand, depending on their current stance. The duration of the timer alternates between the time spent sitting and the time spent standing.

### Usage

```
get-up
```

The timer starts as sitting by default.

#### Controls

- <Tab> and <Shift+Tab> to change selection to next/previous block
- <Space> pauses or resumes the timer
- While the timer block is selected
    - <H> reset the current timer to zero
    - <L> skip directly to the next stance (sit/stand)
- While a setting block is selected
    - <H> decreases the selected stance duration by 5 minutes
    - <L> increases the selected stance duration by 5 minutes