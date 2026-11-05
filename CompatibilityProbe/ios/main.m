#include <SDL.h>
#include <SDL_main.h>
#include <stdbool.h>
#include <stdint.h>
#import <UIKit/UIKit.h>
#import <OpenGLES/EAGL.h>
#import <QuartzCore/QuartzCore.h>
#import <objc/runtime.h>
#include <math.h>
#include <sys/resource.h>
#include <sys/utsname.h>
#include <mach/mach.h>

extern int minecraft_probe_run(const char *, const char *, const char *);

static unsigned int phoneHeight;
static BOOL cropToFill;
// Choose the guest device before boot. Minecraft sizes its HUD, framebuffer
// and touch targets together, and cannot switch device families mid-world.
bool minecraft_uses_ipad_layout(void) {
    static bool initialized, tablet;
    if (!initialized) {
        NSString *layout = [NSUserDefaults.standardUserDefaults stringForKey:@"MinecraftGameLayout"];
        if ([layout isEqualToString:@"ipad"]) tablet = true;
        else if (![layout isEqualToString:@"iphone"]) {
            struct utsname hardware;
            NSString *model = uname(&hardware) == 0 ? @(hardware.machine) : nil;
#if TARGET_OS_SIMULATOR
            model = NSProcessInfo.processInfo.environment[@"SIMULATOR_MODEL_IDENTIFIER"];
#endif
            tablet = UIDevice.currentDevice.userInterfaceIdiom == UIUserInterfaceIdiomPad ||
                     [model isEqualToString:@"iPhone19,4"]; // iPhone Duo
        }
        initialized = true;
        NSLog(@"Minecraft guest layout: %@", tablet ? @"iPad (1024x768)" : @"iPhone");
    }
    return tablet;
}
unsigned int minecraft_phone_height(void) {
    if (!phoneHeight) {
        NSUserDefaults *defaults = NSUserDefaults.standardUserDefaults;
        [defaults registerDefaults:@{@"MinecraftPhoneScreen": @"4", @"FillScreenCropped": @NO, @"ShowFPS": @YES, @"TerrainProcessing": @"background120"}];
        phoneHeight = [[defaults stringForKey:@"MinecraftPhoneScreen"] isEqualToString:@"3.5"] ? 480 : 568;
        cropToFill = [defaults boolForKey:@"FillScreenCropped"];
        NSLog(@"Minecraft display preferences: height=%u crop=%d", phoneHeight, cropToFill);
    }
    return phoneHeight;
}

bool minecraft_native_resolution(void) {
    id enabled = [NSUserDefaults.standardUserDefaults objectForKey:@"NativeGameResolution"];
    return enabled ? [enabled boolValue] : true;
}
bool minecraft_expand_canvas(void) {
    // A previously selected phone crop remains an explicit display choice.
    if (cropToFill && !minecraft_uses_ipad_layout()) return false;
    id enabled = [NSUserDefaults.standardUserDefaults objectForKey:@"ExpandWorldView"];
    return enabled ? [enabled boolValue] : true;
}

// Read at launch, alongside the guest display settings.
static unsigned int backgroundFrameLimit;
unsigned int minecraft_background_frame_limit(void) {
    NSString *mode = [NSUserDefaults.standardUserDefaults stringForKey:@"TerrainProcessing"];
    if ([mode isEqualToString:@"original"]) return 0;
    if ([mode isEqualToString:@"background30"]) return backgroundFrameLimit = 30;
    if ([mode isEqualToString:@"background120"])
        return backgroundFrameLimit = MIN(120, MAX(1, UIScreen.mainScreen.maximumFramesPerSecond));
    return backgroundFrameLimit = 60;
}

@interface EAGLContext (MinecraftSDLDrawable)
@property(nonatomic,readonly) UIView *sdlView;
@end
static BOOL (*originalPresent)(id,SEL,NSUInteger);
static UIView *minecraftUpdateDrawable(id context) {
    static BOOL loggedPresent;
    if (!loggedPresent) {
        NSLog(@"Minecraft native present: class=%@ drawable=%d",[context class],[context respondsToSelector:@selector(sdlView)]);
        loggedPresent=YES;
    }
    if ([context respondsToSelector:@selector(sdlView)]) {
        UIView *view = [context sdlView];
        UIWindow *window = view.window;
        static BOOL loggedView;
        if (!loggedView) {
            NSLog(@"Minecraft native drawable: view=%@ window=%@ scene=%@",view,window,window.windowScene);
            loggedView=YES;
        }
        if (window && !window.windowScene) {
            for (UIScene *scene in UIApplication.sharedApplication.connectedScenes) {
                if ([scene isKindOfClass:UIWindowScene.class] && scene.activationState != UISceneActivationStateUnattached && scene.activationState != UISceneActivationStateBackground) {
                    window.windowScene = (UIWindowScene *)scene;
                    id delegate = scene.delegate;
                    if ([delegate respondsToSelector:@selector(window)] && [delegate window] != window) [[delegate window] setHidden:YES];
                    if ([delegate respondsToSelector:@selector(setWindow:)]) [delegate setWindow:window];
                    [window makeKeyAndVisible];
                    break;
                }
            }
        }
        if (view.superview && window) {
            // Crop only the presented drawable. UIKit inverts this transform
            // before SDL converts touch coordinates into guest coordinates.
            CGSize canvas = view.superview.bounds.size;
            if (canvas.width > 0 && canvas.height > 0) {
                if (!CGSizeEqualToSize(view.bounds.size,canvas)) {
                    view.transform = CGAffineTransformIdentity;
                    view.bounds = (CGRect){CGPointZero,canvas};
                    view.center = CGPointMake(CGRectGetMidX(view.superview.bounds),CGRectGetMidY(view.superview.bounds));
                    [view setNeedsLayout];
                }
                double aspect = minecraft_uses_ipad_layout() ? 1024.0/768.0 : minecraft_phone_height()/320.0;
                // Minecraft stays landscape even when Duo's host window is
                // taller than it is wide. Match Window::viewport rounding.
                double width = round(canvas.width * view.contentScaleFactor);
                double height = round(canvas.height * view.contentScaleFactor);
                double fittedW = width/height > aspect ? round(height*aspect) : width;
                double fittedH = width/height > aspect ? height : round(width/aspect);
                CGFloat zoom = cropToFill && !minecraft_uses_ipad_layout() && fittedW > 0 && fittedH > 0
                    ? fmax(width/fittedW,height/fittedH) : 1.0;
                CGAffineTransform transform = CGAffineTransformMakeScale(zoom,zoom);
                if (!CGAffineTransformEqualToTransform(view.transform,transform)) {
                    view.transform = transform;
                }
                // A layer write opens an implicit transaction, so skip it once set.
                if (!view.superview.clipsToBounds) view.superview.clipsToBounds = YES;
                static CGSize lastCanvas;
                static CGFloat lastZoom;
                if (!CGSizeEqualToSize(lastCanvas,canvas) || lastZoom != zoom) {
                    NSLog(@"Minecraft presentation: canvas=%.0fx%.0f zoom=%.5f",canvas.width,canvas.height,zoom);
                    lastCanvas=canvas;
                    lastZoom=zoom;
                }
            }
        }
    }
    return [context respondsToSelector:@selector(sdlView)] ? [context sdlView] : nil;
}
static CALayer *nativeCanvas, *nativeImage;
extern void minecraft_refresh_sample(double seconds, uint32_t callbacks, uint32_t distinct);
static uint64_t submittedImages;
static CFTimeInterval lastRefreshCallback, nextRefreshTarget;
static uint64_t refreshSequence;
uint64_t minecraft_refresh_deadline(double *seconds) {
    CFTimeInterval delay = nextRefreshTarget - CACurrentMediaTime();
    if (!refreshSequence || delay < -0.04 || delay > 0.1) return 0;
    *seconds = fmax(0.001, delay);
    return refreshSequence;
}
static BOOL profileRefresh;
static UILabel *fpsLabel;
@interface MinecraftRefreshObserver : NSObject
@end
@implementation MinecraftRefreshObserver
- (void)refresh:(CADisplayLink *)link {
    static uint64_t lastImage;
    static uint32_t callbacks, distinct;
    static CFTimeInterval start;
    CFTimeInterval now = link.timestamp;
    nextRefreshTarget = link.targetTimestamp;
    refreshSequence++;
    BOOL changed = lastImage != submittedImages;
    if (changed && backgroundFrameLimit > 60) [CATransaction flush];
    lastImage = submittedImages;
    if (fpsLabel) {
        static CFTimeInterval sampleStart;
        static unsigned sampleFrames;
        if (!sampleStart || now - lastRefreshCallback > 1.0) {
            sampleStart = now;
            sampleFrames = 0;
        } else {
            sampleFrames += changed;
            if (now - sampleStart >= 0.5) {
                fpsLabel.text = [NSString stringWithFormat:@"%.0f FPS", sampleFrames / (now - sampleStart)];
                sampleStart = now;
                sampleFrames = 0;
            }
        }
    }
    if (profileRefresh) {
        if (!start || now - lastRefreshCallback > 1.0) {
            start = now;
            callbacks = distinct = 0;
        } else {
            callbacks++;
            distinct += changed;
            if (now - start >= 5.0) {
                minecraft_refresh_sample(now - start, callbacks, distinct);
                start = now;
                callbacks = distinct = 0;
            }
        }
    }
    lastRefreshCallback = now;
}
@end
static void minecraftConfigureRefresh(UIView *view) {
    static BOOL configured;
    if (configured || !view.window) return;
    configured = YES;
    profileRefresh = [NSProcessInfo.processInfo.environment[@"MINECRAFT_PROFILE"] isEqualToString:@"1"];
    if ([NSUserDefaults.standardUserDefaults boolForKey:@"ShowFPS"]) {
        UIWindow *window = view.window;
        fpsLabel = [UILabel new];
        fpsLabel.translatesAutoresizingMaskIntoConstraints = NO;
        fpsLabel.userInteractionEnabled = NO;
        fpsLabel.accessibilityIdentifier = @"MinecraftFPS";
        fpsLabel.font = [UIFont monospacedDigitSystemFontOfSize:12 weight:UIFontWeightSemibold];
        fpsLabel.textColor = UIColor.whiteColor;
        fpsLabel.backgroundColor = [UIColor.blackColor colorWithAlphaComponent:0.68];
        fpsLabel.textAlignment = NSTextAlignmentCenter;
        fpsLabel.text = @"… FPS";
        fpsLabel.layer.cornerRadius = 5;
        fpsLabel.clipsToBounds = YES;
        [window addSubview:fpsLabel];
        [NSLayoutConstraint activateConstraints:@[
            [fpsLabel.leadingAnchor constraintEqualToAnchor:window.safeAreaLayoutGuide.leadingAnchor constant:6],
            [fpsLabel.topAnchor constraintEqualToAnchor:window.safeAreaLayoutGuide.topAnchor constant:6],
            [fpsLabel.widthAnchor constraintEqualToConstant:61],
            [fpsLabel.heightAnchor constraintEqualToConstant:22],
        ]];
    }
    // Background terrain modes follow this link's vsync deadlines rather than
    // sleep-based pacing, so create it whenever one of them is active.
    if (!backgroundFrameLimit && !profileRefresh && !fpsLabel) return;
    UIScreen *screen = view.window.windowScene.screen ?: UIScreen.mainScreen;
    CADisplayLink *link = [screen displayLinkWithTarget:[[MinecraftRefreshObserver new] autorelease] selector:@selector(refresh:)];
    float target = MIN(MAX(60, backgroundFrameLimit), screen.maximumFramesPerSecond);
    link.preferredFrameRateRange = CAFrameRateRangeMake(MIN(30, target), target, target);
    [link addToRunLoop:NSRunLoop.mainRunLoop forMode:NSRunLoopCommonModes];
}
static BOOL minecraftPresent(id context, SEL selector, NSUInteger target) {
    UIView *view = minecraftUpdateDrawable(context);
    BOOL presented = originalPresent(context, selector, target);
    if (presented && view.window && NSThread.isMainThread && (!nativeCanvas || nativeCanvas.hidden)) {
        minecraftConfigureRefresh(view);
        submittedImages++;
    }
    return presented;
}
void minecraft_native_present_hide(void) {
    if (NSThread.isMainThread) nativeCanvas.hidden = YES;
}
// All calls occur on the existing UIKit/SDL main thread. CGImage owns a copy
// of the pixels until Core Animation finishes with that frame.
bool minecraft_native_present_rgba(const uint8_t *pixels, uint32_t width, uint32_t height,
                                   uint32_t vx, uint32_t vy, uint32_t vw, uint32_t vh) {
    @autoreleasepool {
    if (!NSThread.isMainThread || !pixels || !width || !height || width>4096 || height>4096) return false;
    UIView *view=minecraftUpdateDrawable(EAGLContext.currentContext);
    if (!view.window || view.contentScaleFactor<=0) return false;
    CGFloat scale=view.contentScaleFactor;
    CGFloat drawableH=round(view.bounds.size.height*scale);
    if (!vw || !vh || vx/scale>view.bounds.size.width || vy+vh>drawableH+1) return false;
    CFDataRef data=CFDataCreate(kCFAllocatorDefault,pixels,(CFIndex)width*height*4);
    if (!data) return false;
    CGDataProviderRef provider=CGDataProviderCreateWithCFData(data);
    CGColorSpaceRef color=CGColorSpaceCreateWithName(kCGColorSpaceSRGB);
    if (!provider || !color) {
        if (provider) CGDataProviderRelease(provider);
        if (color) CGColorSpaceRelease(color);
        CFRelease(data);
        return false;
    }
    CGImageRef image=CGImageCreate(width,height,8,32,(size_t)width*4,color,
        kCGBitmapByteOrder32Big|kCGImageAlphaNoneSkipLast,provider,NULL,true,kCGRenderingIntentDefault);
    CGColorSpaceRelease(color);CGDataProviderRelease(provider);CFRelease(data);
    if (!image) return false;
    [CATransaction begin];[CATransaction setDisableActions:YES];
    if (!nativeCanvas) {
        nativeCanvas=[CALayer new];nativeImage=[CALayer new];
        nativeCanvas.backgroundColor=UIColor.blackColor.CGColor;nativeCanvas.opaque=YES;
        nativeImage.opaque=YES;nativeImage.minificationFilter=kCAFilterLinear;nativeImage.magnificationFilter=kCAFilterLinear;
        [nativeCanvas addSublayer:nativeImage];
        NSLog(@"Minecraft native Core Animation presentation enabled");
    }
    if (nativeCanvas.superlayer!=view.layer) {
        [nativeCanvas removeFromSuperlayer];[view.layer insertSublayer:nativeCanvas atIndex:0];
    }
    nativeCanvas.frame=view.bounds;nativeCanvas.hidden=NO;
    nativeImage.frame=CGRectMake(vx/scale,(drawableH-vy-vh)/scale,vw/scale,vh/scale);
    nativeImage.contents=(id)image;
    [CATransaction commit];
    minecraftConfigureRefresh(view);
    submittedImages++;
    if (backgroundFrameLimit <= 60 || CACurrentMediaTime() - lastRefreshCallback > 0.05)
        [CATransaction flush];
    CGImageRelease(image);
    return true;
    }
}

@interface MinecraftProbeScene : NSObject <UIWindowSceneDelegate>
@property(nonatomic,strong) UIWindow *window;
@end
@implementation MinecraftProbeScene
- (void)scene:(UIScene *)scene willConnectToSession:(UISceneSession *)session options:(UISceneConnectionOptions *)options {
    if (![scene isKindOfClass:UIWindowScene.class]) return;
    self.window = [[UIWindow alloc] initWithWindowScene:(UIWindowScene *)scene];
    self.window.rootViewController = [UIViewController new];
    self.window.rootViewController.view.backgroundColor = UIColor.blackColor;
    [self.window makeKeyAndVisible];
}
@end

// Probe integration: sibling-only UI, telemetry, adaptive pacing and cloud
// saves are disabled. These are host callbacks, not replacement guest APIs.
// Keyboard, lifecycle and rendering validation remain later milestones.
// Layout must match diagnostics::Native. Sample only when profiling is enabled.
void peggle_native_diagnostics(void *out) {
    struct NativeSample { double cpu_seconds; uint64_t resident_bytes;
        int32_t thermal, low_power, max_fps, battery_state, orientation,
                drawable_width, drawable_height; } *n=out;
    memset(n,0,sizeof(*n));
    struct rusage usage;
    if (getrusage(RUSAGE_SELF,&usage)==0) {
        n->cpu_seconds=usage.ru_utime.tv_sec+usage.ru_stime.tv_sec+
            (usage.ru_utime.tv_usec+usage.ru_stime.tv_usec)/1000000.0;
    }
    mach_task_basic_info_data_t info;
    mach_msg_type_number_t count=MACH_TASK_BASIC_INFO_COUNT;
    if (task_info(mach_task_self(),MACH_TASK_BASIC_INFO,(task_info_t)&info,&count)==KERN_SUCCESS)
        n->resident_bytes=info.resident_size;
    n->thermal=(int32_t)NSProcessInfo.processInfo.thermalState;
    n->low_power=NSProcessInfo.processInfo.lowPowerModeEnabled;
    n->max_fps=(int32_t)UIScreen.mainScreen.maximumFramesPerSecond;
    n->battery_state=(int32_t)UIDevice.currentDevice.batteryState;
    n->orientation=(int32_t)UIDevice.currentDevice.orientation;
    SDL_Window *window=SDL_GetWindowFromID(1);
    if (window) SDL_GL_GetDrawableSize(window,&n->drawable_width,&n->drawable_height);
}
void peggle_ios_keyboard_native_diagnostic(int32_t stage,uint32_t current,uint32_t replacement) {}
bool peggle_poll_keyboard_event(void *event) { return false; }
void peggle_fps_begin_game_present(void) {}
void peggle_fps_end_game_present(void) {}
void peggle_fps_present_result(void *view, int presented) {}
int peggle_wrapper_backing_metrics(void *window, uint32_t *w, uint32_t *h,
 uint32_t *pw, uint32_t *ph, double *scale, int *status, uint32_t *color) { return 0; }
void peggle_pacing_configure(int value) { NSCAssert(value == 0, @"Peggle pacing must remain disabled"); }
int peggle_pacing_target(int loading) { return 60; }
void peggle_pacing_sample(double seconds, int loading) {}
double peggle_pacing_observed(void) { return 0; }
void peggle_pacing_composition(double seconds, int loading) {}
void peggle_pacing_begin(void) {}
void peggle_pacing_wait(double credit) {}
void peggle_pacing_status(void *out) {}
uint32_t peggle_auto_capture_due(void) { return 0; }
uint32_t peggle_auto_profile(void) { return 0; }
char *peggle_progress_capture(uint32_t id) { return NULL; }
void peggle_auto_snapshot(const char *json) {}
void peggle_save_manager_free(char *json) { free(json); }
int peggle_auto_prompt_due(void) { return 0; }
int peggle_auto_prompt(void) { return 0; }

int SDL_main(int argc, char *argv[]) {
    @autoreleasepool {
        NSBundle *bundle = NSBundle.mainBundle;
        minecraft_phone_height();
        Method present = class_getInstanceMethod(EAGLContext.class, @selector(presentRenderbuffer:));
        originalPresent = (void *)method_getImplementation(present);
        method_setImplementation(present, (IMP)minecraftPresent);
        NSString *documents = NSSearchPathForDirectoriesInDomains(NSDocumentDirectory, NSUserDomainMask, YES).firstObject;
        int result = minecraft_probe_run([bundle pathForResource:@"Minecraft" ofType:@"ipa"].UTF8String,
                                         bundle.resourcePath.UTF8String, documents.UTF8String);
        NSLog(@"MINECRAFT_PROBE_RESULT=%d", result);
        [@(result).stringValue writeToFile:[documents stringByAppendingPathComponent:@"startup-result.txt"]
                                atomically:YES encoding:NSUTF8StringEncoding error:nil];
        return result;
    }
}
