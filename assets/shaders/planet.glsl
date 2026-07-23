uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;
uniform mat3 u_model_matrix;
uniform float u_time;
varying vec2 v_quad_pos;
varying vec2 v_vt;

#ifdef VERTEX_SHADER
attribute vec2 a_pos;
attribute vec2 a_vt;

void main() {
    v_vt = a_vt;

    v_quad_pos = a_pos;
    vec3 pos = u_projection_matrix * u_view_matrix * u_model_matrix * vec3(a_pos, 1.0);
    gl_Position = vec4(pos.xy, 0.0, pos.z);
}
#endif

#ifdef FRAGMENT_SHADER

#define OCTAVES 6

uniform vec4 u_color;

const float planet_size = 0.3;
const float sky_size = 0.35;

const float ground_coverage = 0.58;
const float sky_coverage = 0.55;

const vec3 ocean_col  = vec3(0.2, 0.5, 0.8);
const vec3 ground_col = vec3(0.4, 0.98, 0.6);
const vec3 cloud_col  = vec3(0.98);
const vec3 fresnel_col = vec3(0.1, 0.3, 0.9);
const vec3 atmosphere_col = vec3(0.3, 0.5, 0.9);
const float atmosphere_intensity = 1.75;

const vec3 sun = vec3(1.0);

// subtractive so its weird
const vec3 sun_col = vec3(0.9, 0.5, 0.3);

float random2(vec2 p)
{
    return fract(
        sin(dot(p, vec2(12.9898, 78.233))) *
        43758.5453123
    );
}

float noise2(vec2 p)
{
    vec2 i = floor(p);
    vec2 f = fract(p);

    float a = random2(i);
    float b = random2(i + vec2(1.0, 0.0));
    float c = random2(i + vec2(0.0, 1.0));
    float d = random2(i + vec2(1.0, 1.0));

    vec2 u = f * f * (3.0 - 2.0 * f);

    return mix(a, b, u.x)
         + (c - a) * u.y * (1.0 - u.x)
         + (d - b) * u.x * u.y;
}

float fbm2(vec2 p)
{
    float value = 0.0;
    float amplitude = 0.5;

    for (int i = 0; i < OCTAVES; i++) {
        value += amplitude * noise2(p);
        p *= 2.0;
        amplitude *= 0.5;
    }

    return value;
}

float random3(vec3 p)
{
    return fract(
        sin(dot(p, vec3(12.9898, 78.233, 37.719))) *
        43758.5453123
    );
}

float noise3(vec3 p)
{
    vec3 i = floor(p);
    vec3 f = fract(p);
    vec3 u = f * f * (3.0 - 2.0 * f);

    float n000 = random3(i + vec3(0.0, 0.0, 0.0));
    float n100 = random3(i + vec3(1.0, 0.0, 0.0));
    float n010 = random3(i + vec3(0.0, 1.0, 0.0));
    float n110 = random3(i + vec3(1.0, 1.0, 0.0));

    float n001 = random3(i + vec3(0.0, 0.0, 1.0));
    float n101 = random3(i + vec3(1.0, 0.0, 1.0));
    float n011 = random3(i + vec3(0.0, 1.0, 1.0));
    float n111 = random3(i + vec3(1.0, 1.0, 1.0));

    float bottom = mix(
        mix(n000, n100, u.x),
        mix(n010, n110, u.x),
        u.y
    );

    float top = mix(
        mix(n001, n101, u.x),
        mix(n011, n111, u.x),
        u.y
    );

    return mix(bottom, top, u.z);
}

float fbm3(vec3 p)
{
    float value = 0.0;
    float amplitude = 0.5;

    for (int i = 0; i < OCTAVES; i++) {
        value += amplitude * noise3(p);
        p *= 2.0;
        amplitude *= 0.5;
    }

    return value;
}

float ease_out_cubic(float x) {
    return 1.0 - pow(1.0 - x, 3.0);
}

vec3 sphere(vec2 uv, float radius, float side)
{
    vec2 p = uv / radius;
    float z = sqrt(max(0.0, 1.0 - dot(p, p)));

    return normalize(vec3(p, side * z));
}

vec3 spin(vec3 p, float angle)
{
    float c = cos(angle);
    float s = sin(angle);

    return vec3(
         c * p.x + s * p.z,
         p.y,
        -s * p.x + c * p.z
    );
}

// yoinked from caverim
vec3 fresnel(float amount, float intensity, vec3 color, vec3 normal, vec3 view)
{
	return pow((1.0 - dot(normalize(normal), normalize(view))), amount) * color * intensity;
}

void main() {
    if (length(v_quad_pos) > 1.0) {
        discard;
    }

    vec2 uv = v_vt - 0.5;
    // float pixelate = 4.0 / iResolution.y;
    // uv = floor(uv / pixelate) * pixelate;

    float dist = length(uv);
    
    float planet_circle = 1.0 - smoothstep(
        planet_size,
        planet_size,
        dist
    );

    float sky_circle = 1.0 - smoothstep(
        sky_size,
        sky_size,
        dist
    );

    float rotation = u_time * 0.05;

    float ground_noise = fbm2(uv*7.0+vec2(rotation,0.0));
    float ground = step(0.55, ground_noise);

    vec3 planet_normal = sphere(uv, planet_size, 1.0);
    vec3 light = sun_col*step(dot(planet_normal, sun), 0.5);
    vec3 light_sky = sun_col*step(dot(sphere(uv, sky_size, 1.0), sun), 0.5);
    
    vec3 fresnel = fresnel(1.6, 0.9, fresnel_col, planet_normal, vec3(0.0, 0.0, 1.0));
    
    vec3 planet = mix(
        ocean_col - light * 0.5 + fresnel,
        ground_col - light * 0.8 + fresnel,
        ground
    );

    vec3 front_pos = sphere(uv, sky_size, 1.0);
    vec3 back_pos  = sphere(uv, sky_size, -1.0);

    front_pos = spin(front_pos, rotation);
    back_pos  = spin(back_pos, rotation);

    float front_clouds = step(
        sky_coverage,
        fbm3(front_pos * 3.5)
    );

    float back_clouds = step(
        sky_coverage,
        fbm3(back_pos * 3.5)
    );

    vec4 final = vec4(0.0);
    
    //atmosphere
    final = mix(
        final,
        vec4(atmosphere_col, 1.0),
        max(0.0, ease_out_cubic(0.6-dist/(planet_size*2.5))) * atmosphere_intensity
    );
    
    final = mix(
        final,
        vec4(cloud_col, 1.0),
        sky_circle * back_clouds
    );

    final = mix(
        final,
        vec4(planet, 1.0),
        planet_circle
    );

    final = mix(
        final,
        vec4(cloud_col - light_sky * 0.5, 1.0),
        sky_circle * front_clouds
    );
    
    
    gl_FragColor = final;
}
#endif

