# Celeste Save Data Analyzer

# Celesteとは？

> 大丈夫、きっとやりとげられる

[Steam：Celeste](https://store.steampowered.com/app/504230/Celeste/?l=japanese)

# 構成要素

- `celeste_save_data_rs`: Celesteのセーブデータをパース
- `celeste_visualizer`: Celesteのセーブデータを画像に可視化
- `celeste_bot_rs`: CelesteのセーブデータをDiscord上で受け取り、画像化するbot

# `maps.yaml`へのデータ追加方法

## `maps.yaml`とセーブデータの関係

```yaml
# LevelSetStatsのNameに対応
# 各面のグループIDと思って良い
- level: 'StrawberryJam2021/1-Beginner' 
  # グループの名前 好きに決められる
  name: 'Strawberry Jam Beginner' 
  maps:
    # AreaStatsのSIDに対応
    - sid: 'StrawberryJam2021/1-Beginner/asteriskblue'
      # enは必須 jaは任意
      name:
        en: 'Forest Path'
        ja: '森の小径'
      # 利用するAreaModeStatsのインデックス
      # 例えば、Collab系のステージはB,C面が存在しないのでこのようになる
      # `Celeste/1-ForsakenCity`の場合は`[0, 1, 2]`となっている
      sides: [0]
```

```xml
    <LevelSetStats Name="StrawberryJam2021/1-Beginner">
      <Areas>
        <AreaStats ID="152" Cassette="false" SID="StrawberryJam2021/1-Beginner/asteriskblue">
          <Modes>
            <AreaModeStats TotalStrawberries="0" Completed="false" SingleRunCompleted="false" FullClear="false" Deaths="0" TimePlayed="0" BestTime="0" BestFullClearTime="0" BestDashes="0" BestDeaths="0" HeartGem="false">
              <Strawberries />
              <Checkpoints />
            </AreaModeStats>
            <AreaModeStats TotalStrawberries="0" Completed="false" SingleRunCompleted="false" FullClear="false" Deaths="0" TimePlayed="0" BestTime="0" BestFullClearTime="0" BestDashes="0" BestDeaths="0" HeartGem="false">
              <Strawberries />
              <Checkpoints />
            </AreaModeStats>
            <AreaModeStats TotalStrawberries="0" Completed="false" SingleRunCompleted="false" FullClear="false" Deaths="0" TimePlayed="0" BestTime="0" BestFullClearTime="0" BestDashes="0" BestDeaths="0" HeartGem="false">
              <Strawberries />
              <Checkpoints />
            </AreaModeStats>
          </Modes>
        </AreaStats>
```

## テンプレートについて

`LevelSetStats`の`Name`の文字列が分かれば、`~template <Name>`というコマンドとセーブデータを一緒に投げれば作成できる。

## ステージ名の探し方

Collab系では、sidがステージ制作者名になっていることがあり非常に探しづらい。が、modのファイルを参照することで少しは楽になる。

`<modのzipファイル>/Dialog/English.txt`に、modのに使われる文章全てが書き込まれている。ここに、`<mod名>_<グループ名>_<作者名>`という形でステージ名が書かれていることが多い。この名前をテキストエディタなどで検索してステージ名を検索すると良い。

Strawberry Jam Collabの一部抜粋

```
StrawberryJam2021_1_Beginner_asteriskblue=
  Forest Path
StrawberryJam2021_1_Beginner_asteriskblue_author=
  by Asterisk
StrawberryJam2021_1_Beginner_asteriskblue_collabcredits=
  Music:{# 2a2a2a} catapillie{#}
  Sticker:{# 2a2a2a} fionwe{#}
  Playtesting:{# 2a2a2a} ABuffZucchini, Banana 23,
  Bissy, Coffe, PowerAV, Projecteer{#}
  Captain:{# 2a2a2a} Quinnigan{#}
```
