This should do a few things:

We'd like to be able to parse arbitrary rules as Unleash strategies and evaluate them

We'd also like to be able to convert existing strategies into the new format

Sample formats we'd like to support

userWithId WITH USERS 1,2,3

gradualRolloutUserId WITH (percentage > 50 and groupId = AB12A)

gradualRolloutSessionId WITH (percentage > 50 and groupId = AB12A)

gradualRolloutRandom WITH (percentage > 10)

user_id < 5
user_id in [1, 2, 3]
user_id not in [1, 2, 3]


sticky on user_id

50% sticky on user_id with group_id