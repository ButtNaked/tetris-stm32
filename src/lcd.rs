#![allow(dead_code)]

use super::pause;
use stm32f1xx_hal::prelude::*;


#[derive(Copy, Clone)]
pub struct Rect {
    pub x : usize,
    pub y : usize,
    pub w : usize,
    pub h : usize,
}


const LCD_WIDTH : usize = 320;
const LCD_HEIGHT : usize = 240;
const FULL_SCREEN_RECT : Rect = Rect { x : 0, y : 0, w : LCD_WIDTH, h : LCD_HEIGHT };

pub struct Lcd {}

impl Lcd {

    pub fn new() -> Self {
        Lcd {}
    }

    pub fn init(&mut self) {
        use ports::*;
        use io::*;

        // Проверка тактирования GPIO
        lcd_init_check();
        // Снимаем сигнал RES на LCD
        lcd_reset(true);
        // Выбор типа интерфейса 16 бит
        AdrIndex::get().write(0x23);
        // Инициализая контроллера дисплея
        Self::init_disp_controller();
        // Очистка памяти дисплея
        self.clear();
        // Пауза 1 сек
        pause(1000.ms());
        // Включаем питание подсветки
        lcd_pwr(true);
        // Включаем видимость дисплея
        Self::display_enable(true);

    }

    pub fn set_pixel(&mut self, x : usize, y : usize) {

    }

    pub fn clear(&mut self) {
        let rect = FULL_SCREEN_RECT;
        self.fill_rect_with_color(rect, 0b00000_111111_00000);
    }

    /// Заливка области установленным цветом
    pub fn fill_rect_with_color(&mut self, rect : Rect ,color : u16) {
        Self::set_rect(rect);
        Self::fill_with_color(rect.w * rect.h, color);
    }

    /// Инициализация контроллера дисплея
    fn init_disp_controller() {
        use io::*;

        let adr_index = AdrIndex::get();
        let adr_parmt = AdrParmt::get();

        // disp off
        adr_index.write(0x05);
        adr_parmt.write(0x00);

        // RGB interface control
        adr_index.write(0x02);
        adr_parmt.write(0x00);

        // standby_Off
        adr_index.write(0x10);
        adr_parmt.write(0x00);

        adr_index.write(0x30);
        adr_parmt.write(0x00);

        adr_index.write(0x31);
        adr_parmt.write(0x00);

        adr_index.write(0x32);
        adr_parmt.write(0x00);

        Self::set_entry_mode();
    }

    fn set_entry_mode() {
        use io::*;
        let adr_index = AdrIndex::get();
        let adr_parmt = AdrParmt::get();

        adr_index.write(0x03);
        adr_parmt.write(0x30);
    }

    /// команда включения/выключения отображения 
    fn display_enable(on : bool ) {
        use io::*;
        if on {
            // disp on
            AdrIndex::get().write(0x05);
            AdrParmt::get().write(0x01);
        }
        else {
            // disp off
            AdrIndex::get().write(0x05);
            AdrParmt::get().write(0x00);
        }
    }

    /// Установка области развертки
    fn set_rect(rect : Rect) {
        let x1 = rect.x;
        let x2 = rect.x + rect.w - 1;
        let y1 = rect.y;
        let y2 = rect.y + rect.h - 1;

        use io::*;
        let adr_index = AdrIndex::get();
        let adr_parmt = AdrParmt::get();

        adr_index.write(0x35); adr_parmt.write(x1 as u16);
        adr_index.write(0x36); adr_parmt.write(x2 as u16);
        let yy = ((y1 << 8) | y2) as u16;
        adr_index.write(0x37); adr_parmt.write(yy);

        adr_index.write(0x20); adr_parmt.write(y1 as u16);
        adr_index.write(0x21); adr_parmt.write(x1 as u16);
    }

    /// Залитие области развертки цветом
    fn fill_with_color(mut ln : usize, color : u16) {
        use io::*;
        AdrIndex::get().write(0x22);
        while ln != 0  {
            AdrParmt::get().write(color);
            ln -= 1;
        }
    }
}


mod ports {
    use stm32f1xx_hal::pac::{
        RCC,
        GPIOC,
        GPIOF,
    };

    /// Проверяем что тактирование подано
    pub fn lcd_init_check() {
        let rcc = unsafe { &*RCC::ptr() };
        let mut p_rdy = rcc.apb2enr.read().iopcen().is_enabled();
        p_rdy &= rcc.apb2enr.read().iopfen().is_enabled();

        assert!(p_rdy, "Peripheral is not enabled")
    }
    /// Логический сброс
    pub fn lcd_reset(on : bool) {

        let gpioc = unsafe { &*GPIOC::ptr() };
        gpioc.odr.modify(|_, w| {
            w.odr0().bit(on)
        });
    }

    /// Вкл/Выкл всей схемы OLED (питание подсветки)
    pub fn lcd_pwr(on : bool) {

        let gpiof = unsafe { &*GPIOF::ptr() };
        gpiof.odr.modify(|_, w| {
            w.odr15().bit(on)
        });
    }
}

mod io {
    use volatile_register::{RW, WO};

    #[repr(C)]
    pub struct AdrIndex {
        r : RW<u16>,
    }

    #[repr(C)]
    pub struct AdrParmt {
        r : WO<u16>,
    }

    impl AdrIndex {
        pub fn get() -> &'static mut AdrIndex {
            unsafe { &mut *(0x6000_0000 as *mut AdrIndex) }
        }
        pub fn read(&mut self) -> u16 {
            self.r.read()
        }
        pub fn write(&mut self, bb : u16) {
            unsafe { self.r.write(bb) };
        }
    }

    impl AdrParmt {
        pub fn get() -> &'static mut AdrParmt {
            unsafe { &mut *(0x6008_0000 as *mut AdrParmt) }
        }
        pub fn write(&mut self, bb : u16) {
            unsafe { self.r.write(bb) };
        }
    }
}
