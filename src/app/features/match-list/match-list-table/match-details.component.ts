import { Component, computed, inject, signal } from '@angular/core';
import { MatButtonModule } from '@angular/material/button';
import { MAT_DIALOG_DATA, MatDialogModule } from '@angular/material/dialog';
import { MatTabsModule } from '@angular/material/tabs';
import { AgGridAngular } from 'ag-grid-angular';
import { ColDef, GridApi, GridReadyEvent } from 'ag-grid-community';
import { ticks2time } from 'src/app/core/services/tauri.service';
import { ActionInfo, MatchItem, MessageInfo, PlayerInfo } from 'src/types';

type ActionRow = ActionInfo & {
    command?: {
        opcode_name?: string;
        blueprint?: { display_name?: string };
    };
};

const ACTIONS_TAB = 1;

const TEAM_STYLES = [
    { chip: 'bg-blue-100 text-blue-900', text: 'text-blue-800' },
    { chip: 'bg-red-100 text-red-900', text: 'text-red-800' },
];

@Component({
    template: `
        <h1 mat-dialog-title>
            Match Details
            <span class="text-sm font-normal text-gray-500">#{{ data.match_id }}</span>
        </h1>

        <section
            class="flex flex-wrap items-center gap-x-6 gap-y-2 px-6 pb-3 text-sm"
            aria-label="Match summary"
        >
            <div class="flex flex-col">
                <span class="font-medium">{{ mapName }}</span>
                <span class="text-gray-500">{{ data.duration }}</span>
            </div>
            <div class="flex flex-wrap items-center gap-2">
                @for (team of teams; track team.index) {
                    @if (!$first) {
                        <span class="text-gray-500">vs.</span>
                    }
                    <div class="flex flex-wrap items-center gap-1">
                        <span aria-hidden="true">{{ team.won ? '&#127942;' : '&#9760;' }}</span>
                        <span class="sr-only">{{ team.won ? 'Winners:' : 'Defeated:' }}</span>
                        @for (player of team.players; track player.slot) {
                            <span class="rounded-full px-2 py-0.5" [class]="team.style.chip">
                                {{ player.name }}
                            </span>
                        }
                    </div>
                }
            </div>
        </section>

        <mat-dialog-content>
            <mat-tab-group
                animationDuration="0ms"
                [selectedIndex]="selectedTab()"
                (selectedIndexChange)="selectedTab.set($event)"
            >
                <mat-tab [label]="'Messages (' + messages.length.toLocaleString() + ')'">
                    <div class="flex h-full flex-col gap-3 pt-3">
                        <input
                            type="search"
                            [class]="fieldClass"
                            placeholder="Search messages"
                            aria-label="Search messages"
                            #messageSearch
                            [value]="messageQuery()"
                            (input)="messageQuery.set(messageSearch.value)"
                        />
                        @if (messages.length === 0) {
                            <p class="text-gray-500">No messages in this replay.</p>
                        } @else if (filteredMessages().length === 0) {
                            <p class="text-gray-500">No messages match your search.</p>
                        } @else {
                            <ul class="m-0 min-h-0 flex-1 list-none space-y-1 overflow-y-auto p-0" role="log">
                                @for (message of filteredMessages(); track $index) {
                                    <li class="flex gap-3">
                                        <span class="w-12 shrink-0 font-mono text-gray-500">{{ time(message.tick) }}</span>
                                        <span class="min-w-0 break-words">
                                            <span class="font-semibold" [class]="senderClass(message.sender)">{{ message.sender }}</span>
                                            <span class="text-gray-500"> → {{ message.receiver }}:</span>
                                            {{ message.body }}
                                        </span>
                                    </li>
                                }
                            </ul>
                        }
                    </div>
                </mat-tab>

                <mat-tab [label]="'Actions (' + actions.length.toLocaleString() + ')'">
                    <div class="flex h-full flex-col gap-3 pt-3">
                        <div class="flex flex-wrap items-center gap-3">
                            <input
                                type="search"
                                [class]="fieldClass"
                                placeholder="Search actions"
                                aria-label="Search actions"
                                #actionSearch
                                [value]="actionQuery()"
                                (input)="actionQuery.set(actionSearch.value)"
                            />
                            <select
                                [class]="fieldClass"
                                aria-label="Filter actions by player"
                                #player
                                (change)="actionPlayer.set(player.value)"
                            >
                                <option value="" [selected]="actionPlayer() === ''">All players</option>
                                @for (player of data.players; track player.slot) {
                                    <option [value]="player.name" [selected]="actionPlayer() === player.name">
                                        {{ player.name }}
                                    </option>
                                }
                            </select>
                            <span class="text-sm text-gray-500" aria-live="polite">
                                {{ filteredActions().length.toLocaleString() }} shown
                            </span>
                        </div>
                        <ag-grid-angular
                            class="min-h-0 flex-1"
                            style="width: 100%"
                            [rowData]="filteredActions()"
                            [columnDefs]="actions_colDefs"
                            [quickFilterText]="actionQuery()"
                            [overlayNoRowsTemplate]="'No actions match your filters.'"
                            (gridReady)="onActionsGridReady($event)"
                        />
                    </div>
                </mat-tab>
            </mat-tab-group>
        </mat-dialog-content>

        <mat-dialog-actions align="end">
            @if (selectedTab() === ACTIONS_TAB) {
                <button matButton [disabled]="actions.length === 0" (click)="exportActions()">
                    Export CSV
                </button>
            }
            <button matButton mat-dialog-close>Close</button>
        </mat-dialog-actions>
    `,
    styles: `
        :host {
            display: flex;
            flex-direction: column;
            height: 100%;
        }
        mat-dialog-content {
            display: flex;
            flex: 1 1 auto;
            flex-direction: column;
            min-height: 0;
            max-height: none;
            overflow: hidden;
        }
        mat-tab-group {
            flex: 1 1 auto;
            min-height: 0;
        }
        :host ::ng-deep .mat-mdc-tab-body-wrapper {
            flex: 1 1 auto;
            min-height: 0;
        }
        :host ::ng-deep .mat-mdc-tab-body-content {
            height: 100%;
            overflow: hidden;
        }
    `,
    imports: [AgGridAngular, MatDialogModule, MatButtonModule, MatTabsModule],
})
export class MatchDetailsComponent {
    protected data = inject<MatchItem>(MAT_DIALOG_DATA);

    protected messages = this.data.messages ?? [];
    protected actions = (this.data.actions ?? []) as ActionRow[];

    protected mapName = this.data.map.path.replace('DATA:maps\\pvp\\', '') || this.data.map.name;

    protected teams = this.buildTeams(this.data.players);
    private teamByName = new Map(
        this.data.players.map((p) => [p.name, p.team] as const),
    );

    protected readonly ACTIONS_TAB = ACTIONS_TAB;
    protected readonly fieldClass =
        'rounded border border-gray-300 px-3 py-1.5 text-sm focus-visible:outline focus-visible:outline-2 focus-visible:outline-blue-600';

    protected selectedTab = signal(
        this.messages.length > 0 || this.actions.length === 0 ? 0 : ACTIONS_TAB,
    );

    protected messageQuery = signal('');
    protected actionQuery = signal('');
    protected actionPlayer = signal('');

    protected filteredMessages = computed(() => {
        const query = this.messageQuery().trim().toLowerCase();
        if (!query) {
            return this.messages;
        }
        return this.messages.filter((m: MessageInfo) =>
            `${m.sender} ${m.receiver} ${m.body}`.toLowerCase().includes(query),
        );
    });

    protected filteredActions = computed(() => {
        const player = this.actionPlayer();
        return player
            ? this.actions.filter((a) => a.name === player)
            : this.actions;
    });

    private actionsApi?: GridApi<ActionRow>;

    protected actions_colDefs: ColDef<ActionRow>[] = [
        {
            width: 90,
            field: 'tick',
            headerName: 'Time',
            sort: 'asc',
            valueFormatter: (params) => ticks2time(params.data?.tick ?? 0),
            getQuickFilterText: (params) => ticks2time(params.value ?? 0),
        },
        {
            flex: 1,
            field: 'name',
            headerName: 'Player',
            filter: 'agTextColumnFilter',
        },
        {
            flex: 1.5,
            headerName: 'Action',
            valueGetter: (params) => params.data?.command?.opcode_name ?? '',
        },
        {
            flex: 1.5,
            headerName: 'Target',
            valueGetter: (params) =>
                params.data?.command?.blueprint?.display_name ?? '',
        },
        { width: 120, field: 'relic_id', headerName: 'Relic ID' },
    ];

    protected time(tick: number) {
        return ticks2time(tick);
    }

    protected senderClass(sender: string) {
        const team = this.teamByName.get(sender);
        return team === undefined ? '' : TEAM_STYLES[team % TEAM_STYLES.length].text;
    }

    protected onActionsGridReady(event: GridReadyEvent<ActionRow>) {
        this.actionsApi = event.api;
    }

    protected exportActions() {
        this.actionsApi?.exportDataAsCsv({
            fileName: `match-${this.data.match_id}-actions.csv`,
        });
    }

    private buildTeams(players: PlayerInfo[]) {
        const winningTeam = players.find((p) => p.status === 'Won')?.team ?? 0;
        return [0, 1].map((index) => ({
            index,
            won: index === winningTeam,
            style: TEAM_STYLES[index],
            players: players.filter((p) => p.team === index),
        }));
    }
}
