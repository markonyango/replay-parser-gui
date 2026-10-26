import { MatchItem, UploadStatus } from 'src/types';

/** Shape of `status` in matches stored before it became an `UploadStatus`. */
type LegacyStatus = { response?: string; error?: string };

/** A match as read from store.json, which may predate `UploadStatus`. */
export type StoredMatch = Omit<MatchItem, 'status'> & { status: UploadStatus | LegacyStatus };

/** Older store.json entries hold the server's `{response}` / `{error}` body. */
export function migrateStoredMatch(match: StoredMatch): MatchItem {
    const status = match.status;
    if ('kind' in status) {
        return match as MatchItem;
    }

    const migrated: UploadStatus =
        status.response !== undefined
            ? { kind: 'accepted', message: status.response }
            : { kind: 'rejected', message: status.error ?? '' };

    return { ...match, status: migrated };
}

export function uploadStatusText(status: UploadStatus): string {
    switch (status.kind) {
        case 'accepted':
        case 'rejected':
            return status.message;
        case 'failed':
            return status.reason;
        case 'pending':
            return 'pending';
    }
}
