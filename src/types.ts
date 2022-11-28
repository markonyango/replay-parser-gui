export type ReplayInfo = {
  id: number,
  name: string,
  mod_chksum: number,
  mod_version: number,
  md5: string,
  date: string,
  ticks: number,
  game: GameInfo,
  map: MapInfo,
  players: Array<PlayerInfo>
  observers: Array<ObserverInfo>,
  messages: Array<MessageInfo>,
  actions: Array<ActionInfo>
}

type GameInfo = {
  name: string,
  mode: string,
  resources: string,
  locations: string,
  victory_points: number,
};

type MapInfo = {
  name: string,
  description: string,
  abbrname: string,
  maxplayers: number,
  path: string,
  date: string,
  width: number,
  height: number,
};

type PlayerInfo = {
  name: string,
  kind: number,
  team: number,
  race: string,
  relic_id: number,
  rank: number,
  cpu: number,
  hero: number,
  primary_color: number,
  secondary_color: number,
  trim_color: number,
  accessory_color: number,
  skin_path: string,
  skin_name: string,
  id: number,
};

type ObserverInfo = unknown;

type MessageInfo = {
  tick: number,
  sender: string,
  receiver: string,
  body: string,
  player_id: number,
};

type ActionInfo = {
  relic_id: number,
  name: string,
  tick: number,
  data: Array<number>,
}
