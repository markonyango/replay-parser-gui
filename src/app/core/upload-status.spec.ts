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
});

describe('uploadStatusText', () => {
    it('shows the server message or the failure reason', () => {
        expect(uploadStatusText({ kind: 'accepted', message: 'ok' })).toBe('ok');
        expect(uploadStatusText({ kind: 'rejected', message: 'bad replay' })).toBe('bad replay');
        expect(uploadStatusText({ kind: 'failed', reason: 'timeout' })).toBe('timeout');
        expect(uploadStatusText({ kind: 'pending' })).toBe('pending');
    });
});
