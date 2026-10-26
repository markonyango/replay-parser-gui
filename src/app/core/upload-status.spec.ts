import { MatchItem } from 'src/types';
import { migrateStoredMatch, StoredMatch, uploadStatusText } from './upload-status';

function stored(status: StoredMatch['status']): StoredMatch {
    return {
        match_id: 1,
        players: [],
        map: {} as MatchItem['map'],
        duration: '00:00',
        status,
        played_at: '',
        messages: [],
        actions: [],
    };
}

describe('migrateStoredMatch', () => {
    it('keeps matches that already carry an UploadStatus', () => {
        const match = stored({ kind: 'failed', reason: 'timeout' });

        expect(migrateStoredMatch(match)).toBe(match as MatchItem);
    });

    it('maps a legacy server response to accepted', () => {
        expect(migrateStoredMatch(stored({ response: 'ok' })).status).toEqual({
            kind: 'accepted',
            message: 'ok',
        });
    });

    it('maps a legacy server error to rejected', () => {
        expect(migrateStoredMatch(stored({ error: 'bad replay' })).status).toEqual({
            kind: 'rejected',
            message: 'bad replay',
        });
    });

    it('strips raw command fields from stored actions and keeps the rest', () => {
        const match: StoredMatch = {
            ...stored({ kind: 'accepted', message: 'ok' }),
            actions: [
                {
                    relic_id: 1,
                    name: 'Alice',
                    tick: 10,
                    data: [3, 0, 1],
                    command: {
                        opcode_name: 'CMD_BuildSquad',
                        blueprint: { display_name: 'Slugga Boyz' },
                        raw_context: [0, 0, 0, 0],
                        receivers: [{ raw: 1, kind: 0, instance_id: 1 }],
                        custom: { tag: 255, length: null, data: [] },
                    },
                },
                { relic_id: 2, name: 'Bob', tick: 20, data: [], command: null },
            ],
        } as StoredMatch;

        const migrated = migrateStoredMatch(match);

        expect(migrated).not.toBe(match as MatchItem);
        expect(migrated.status).toBe(match.status as MatchItem['status']);
        expect(migrated.actions as unknown[]).toEqual([
            {
                relic_id: 1,
                name: 'Alice',
                tick: 10,
                command: {
                    opcode_name: 'CMD_BuildSquad',
                    blueprint: { display_name: 'Slugga Boyz' },
                },
            },
            { relic_id: 2, name: 'Bob', tick: 20, command: null },
        ]);
    });

    it('keeps already slim actions untouched', () => {
        const match: StoredMatch = {
            ...stored({ kind: 'pending' }),
            actions: [{ relic_id: 1, name: 'Alice', tick: 10, command: null } as StoredMatch['actions'][number]],
        };

        expect(migrateStoredMatch(match)).toBe(match as MatchItem);
    });
});

describe('uploadStatusText', () => {
    it('shows the server message or the failure reason', () => {
        expect(uploadStatusText({ kind: 'accepted', message: 'ok' })).toBe('ok');
        expect(uploadStatusText({ kind: 'rejected', message: 'bad replay' })).toBe('bad replay');
        expect(uploadStatusText({ kind: 'failed', reason: 'timeout' })).toBe('timeout');
        expect(uploadStatusText({ kind: 'pending' })).toBe('pending');
    });
});
