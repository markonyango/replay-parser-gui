import { ComponentFixture, TestBed } from '@angular/core/testing';
import { StatusBarComponent } from './status-bar.component';

describe('StatusBarComponent', () => {
    let fixture: ComponentFixture<StatusBarComponent>;
    let el: HTMLElement;

    beforeEach(() => {
        fixture = TestBed.createComponent(StatusBarComponent);
        el = fixture.nativeElement;
    });

    it('shows the status text', () => {
        fixture.componentRef.setInput('text', 'Watching for replays');
        fixture.detectChanges();

        expect(el.querySelector('[data-testid="status-text"]')?.textContent).toContain(
            'Watching for replays'
        );
    });

    it('renders one item per indicator with label and state', () => {
        fixture.componentRef.setInput('indicators', [
            { id: 'a', label: 'Replay folder', state: 'ok' },
            { id: 'b', label: 'Logfile', state: 'error', tooltip: 'missing' },
            { id: 'c', label: 'Upload', state: 'ok' },
        ]);
        fixture.detectChanges();

        const items = el.querySelectorAll('li');
        expect(items.length).toBe(3);
        expect(items[1].textContent).toContain('Logfile');
        expect(items[0].getAttribute('data-state')).toBe('ok');
        expect(items[1].getAttribute('data-state')).toBe('error');
        expect(items[1].getAttribute('title')).toBe('missing');
        expect(items[0].querySelector('.bg-green-600')).toBeTruthy();
        expect(items[1].querySelector('.bg-red-600')).toBeTruthy();
    });

    it('renders no indicators when none are given', () => {
        fixture.detectChanges();
        expect(el.querySelectorAll('li').length).toBe(0);
    });

    it('is a live status region', () => {
        fixture.detectChanges();
        expect(el.querySelector('[role="status"]')).toBeTruthy();
    });
});
