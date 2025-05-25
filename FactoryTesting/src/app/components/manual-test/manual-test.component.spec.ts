import { ComponentFixture, TestBed } from '@angular/core/testing';

import { ManualTestComponent } from './manual-test.component';

describe('ManualTestComponent', () => {
  let component: ManualTestComponent;
  let fixture: ComponentFixture<ManualTestComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [ManualTestComponent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(ManualTestComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
