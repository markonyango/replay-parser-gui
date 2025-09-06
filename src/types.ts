export type ReplayInfo = {
  id: number;
  name: string;
  mod_chksum: number;
  mod_version: number;
  md5: string;
  date: string;
  ticks: number;
  game: GameInfo;
  map: MapInfo;
  aborted: boolean;
  status: string;
  players: Array<PlayerInfo>;
  messages: Array<MessageInfo>;
  actions: Array<ActionInfo>;
  observers?: Array<ObserverInfo>;
};

export interface MatchItem {
  match_id: number,
  players: PlayerInfo[],
  map: MapInfo,
  duration: string,
  status: string,
  played_at: string,
  messages: MessageInfo[],
  actions: ActionInfo[]
}

export type GameInfo = {
  name: string;
  mode: string;
  resources: string;
  locations: string;
  victory_points: number;
};

export type MapInfo = {
  name: string;
  description: string;
  abbrname: string;
  maxplayers: number;
  path: string;
  date: string;
  width: number;
  height: number;
};

export type PlayerInfo = {
  slot: number;
  steam_id: number;
  sim_id: number;
  status: 'Killed' | 'Dropped' | 'Conceded' | 'Won' | 'Playing';
  name: string;
  kind: number;
  team: number;
  race: number;
  relic_id: number;
  rank: number;
  cpu: number;
  hero: number;
  primary_color: number;
  secondary_color: number;
  trim_color: number;
  accessory_color: number;
  skin_path: string;
  skin_name: string;
  id: number;
};

export type ObserverInfo = unknown;

export type MessageInfo = {
  tick: number;
  sender: string;
  receiver: string;
  body: string;
  player_id: number;
};

export type ActionInfo = {
  relic_id: number;
  name: string;
  tick: number;
  data: Array<number>;
};
