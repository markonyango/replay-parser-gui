import { MapInfo } from 'src/types';

/** The backend serializes `slug`; the path fallback covers matches stored before it existed. */
export function mapSlug(map: MapInfo | undefined): string | undefined {
    return map?.slug ?? map?.path.replace('DATA:maps\\pvp\\', '');
}
