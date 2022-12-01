import { watch, DebouncedEvent } from 'tauri-plugin-fs-watch-api';
import { documentDir } from '@tauri-apps/api/path';
import { invoke } from '@tauri-apps/api';
import { ReplayInfo } from './types';

let stopWatching: any = null;

function updateResponse(returnValue: DebouncedEvent) {
	const { payload: filepath, type } = returnValue;

	if (type === 'Write' && filepath.includes('.rec')) {
		// invoke<string>('parse_file', { path: filepath })
		// 	.then(JSON.parse)
		// 	.then(handleParsedJson)
		// 	.catch(console.error);

		invoke<string>('update_game_list').then(JSON.parse).then(handleParsedJson).catch(console.error);
	}
}

async function _watch() {
	if (stopWatching) {
		await stopWatching();
		stopWatching = null;
	}

	const documentDirPath = await documentDir();
	console.log('Starting to watch: ', documentDirPath);
	stopWatching = await watch(documentDirPath, { recursive: true }, updateResponse).catch((error) =>
		console.error(error)
	);
}

window.addEventListener('DOMContentLoaded', () => _watch());

function handleParsedJson(json: ReplayInfo) {
	const matchListElement = document.querySelector('#matchlist');

	const { id, players, map, ticks, date: match_time } = json;
	console.log(json)
	const map_name = map.path.replace('DATA:maps\\pvp\\', '');

	const teamOne = players.filter(player => player.team === 0);
	const teamTwo = players.filter(player => player.team === 1);

	const rowElement = document.createElement('tr');

	const matchIdElement = document.createElement('td');

	const playersElement = document.createElement('td');
	const teamOneWrapperElement = document.createElement('div');
	const teamTwoWrapperElement = document.createElement('div');

	teamOne.map(player => {
		const playerElement = document.createElement('span');
		playerElement.textContent = player.name;
		return playerElement;
	}).forEach(element => teamOneWrapperElement.appendChild(element));

	teamTwo.map(player => {
		const playerElement = document.createElement('span');
		playerElement.textContent = player.name;
		return playerElement;
	}).forEach(element => teamTwoWrapperElement.appendChild(element));

	playersElement.appendChild(teamOneWrapperElement);
	playersElement.appendChild(teamTwoWrapperElement);


	const mapElement = document.createElement('td');
	const durationElement = document.createElement('td');
	const statusElement = document.createElement('td');
	const timeElement = document.createElement('td');

	matchIdElement.textContent = id.toString();
	// playersElement.textContent = players.map((player) => player.name).join(', ');
	mapElement.textContent = map_name;
	durationElement.textContent = ticks2time(ticks);
	statusElement.textContent = '';
	timeElement.textContent = match_time;

  const rowClassList = [
		'border-b',
		'border-slate-100',
		'dark:border-slate-700',
		'p-4',
		'pl-8',
		'text-slate-500',
		'dark:text-slate-400',
    'whitespace-nowrap',
    'truncate'
	];

	matchIdElement.classList.add(...rowClassList);
  playersElement.classList.add(...rowClassList);
  mapElement.classList.add(...rowClassList);
  durationElement.classList.add(...rowClassList);
  statusElement.classList.add(...rowClassList);
  timeElement.classList.add(...rowClassList);

	rowElement.appendChild(matchIdElement);
	rowElement.appendChild(playersElement);
	rowElement.appendChild(mapElement);
	rowElement.appendChild(durationElement);
	rowElement.appendChild(statusElement);
	rowElement.appendChild(timeElement);

	matchListElement?.appendChild(rowElement);
}

function ticks2time(ticks: number) {
	const total_seconds = Math.floor(ticks / 10);
	const minutes = Math.floor(total_seconds / 60);
	const remaining_seconds = total_seconds - minutes * 60;

	return `${minutes < 10 ? '0' + minutes : minutes}:${
		remaining_seconds < 10 ? '0' + remaining_seconds : remaining_seconds
	}`;
}
