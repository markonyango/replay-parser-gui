import { TestBed } from '@angular/core/testing';
import { StatusService } from './status.service';

describe('StatusService', () => {
    let service: StatusService;

    beforeEach(() => {
        TestBed.configureTestingModule({});
        service = TestBed.inject(StatusService);
    });

    it('sets the text', () => {
        service.setText('Watching');
        expect(service.text()).toBe('Watching');
    });

    it('adds a new indicator', () => {
        service.setIndicator({ id: 'a', label: 'A', state: 'ok' });
        expect(service.indicators().map((i) => i.id)).toEqual(['a']);
    });

    it('replaces an indicator with the same id in place', () => {
        service.setIndicator({ id: 'a', label: 'A', state: 'ok' });
        service.setIndicator({ id: 'b', label: 'B', state: 'ok' });
        service.setIndicator({ id: 'a', label: 'A', state: 'error' });

        expect(service.indicators().map((i) => [i.id, i.state])).toEqual([
            ['a', 'error'],
            ['b', 'ok'],
        ]);
    });

    it('removes an indicator', () => {
        service.setIndicator({ id: 'a', label: 'A', state: 'ok' });
        service.removeIndicator('a');
        expect(service.indicators()).toEqual([]);
    });

    it('applies a full snapshot', () => {
        service.setIndicator({ id: 'old', label: 'Old', state: 'ok' });
        service.apply({
            text: 'Hi',
            indicators: [{ id: 'new', label: 'New', state: 'error' }],
        });

        expect(service.text()).toBe('Hi');
        expect(service.indicators().map((i) => i.id)).toEqual(['new']);
    });
});
