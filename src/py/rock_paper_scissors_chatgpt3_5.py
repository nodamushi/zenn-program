#!python3
# ChatGPT3.5 が作った複数人で行うじゃんけんプログラム。ひどい。
num_players = int(input("プレイヤーの人数を入力してください: "))

players = []
for i in range(num_players):
    name = input(f"プレイヤー{i+1}の名前を入力してください: ")
    hand = input("グー、チョキ、パーのいずれかを入力してください: ")
    while hand not in ["グー", "チョキ", "パー"]:
        hand = input("グー、チョキ、パーのいずれかを入力してください: ")
    players.append((name, hand))

winners = []
for i in range(len(players)):
    print(f"{players[i][0]}さんの手: {players[i][1]}")
for hand in ["グー", "チョキ", "パー"]:
    hand_players = [p for p in players if p[1] == hand]
    if len(hand_players) == 0:
        continue
    elif len(hand_players) == 1:
        winner = hand_players[0][0]
    else:
        print(f"{hand}のあいこです")
        continue
    print(f"{hand}を出したプレイヤーは{[p[0] for p in hand_players]}で、{winner}さんが勝ちました")
    winners.append(winner)

print("優勝者は以下のとおりです")
for winner in winners:
    print(winner)
